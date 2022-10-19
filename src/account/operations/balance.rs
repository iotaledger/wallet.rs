// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::block::output::{unlock_condition::UnlockCondition, NativeTokensBuilder, Output, Rent};
use primitive_types::U256;

use crate::account::{
    handle::AccountHandle,
    operations::helpers::time::can_output_be_unlocked_forever_from_now_on,
    types::{AccountBalance, BaseCoinBalance, NativeTokensBalance},
    OutputsToClaim,
};

impl AccountHandle {
    /// Get the AccountBalance
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        log::debug!("[BALANCE] get balance");
        let unlockable_outputs_with_multiple_unlock_conditions = self
            .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToClaim::All)
            .await?;

        let account_addresses = self.addresses().await?;
        let account = self.read().await;

        let network_id = self.client.get_network_id()?;
        let rent_structure = self.client.get_rent_structure()?;

        let local_time = self.client.get_time_checked()?;

        let mut total_amount = 0;
        let mut required_storage_deposit = 0;
        let mut total_native_tokens = NativeTokensBuilder::new();
        let mut potentially_locked_outputs = HashMap::new();
        let mut aliases = Vec::new();
        let mut foundries = Vec::new();
        let mut nfts = Vec::new();

        for output_data in account.unspent_outputs.values() {
            // Check if output is from the network we're currently connected to
            if output_data.network_id != network_id {
                continue;
            }

            // Add alias and foundry outputs here because they can't have a [`StorageDepositReturnUnlockCondition`]
            // or time related unlock conditions
            match &output_data.output {
                Output::Alias(output) => {
                    // Add amount
                    total_amount += output_data.output.amount();
                    // Add storage deposit
                    required_storage_deposit += &output_data.output.rent_cost(&rent_structure);
                    // Add native tokens
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        total_native_tokens.add_native_tokens(native_tokens.clone())?;
                    }
                    let alias_id = output.alias_id().or_from_output_id(output_data.output_id);
                    aliases.push(alias_id);
                }
                Output::Foundry(output) => {
                    // Add amount
                    total_amount += output_data.output.amount();
                    // Add storage deposit
                    required_storage_deposit += &output_data.output.rent_cost(&rent_structure);
                    // Add native tokens
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        total_native_tokens.add_native_tokens(native_tokens.clone())?;
                    }
                    foundries.push(output.id())
                }
                _ => {
                    // If there is only an [AddressUnlockCondition], then we can spend the output at any time without
                    // restrictions
                    if let [UnlockCondition::Address(_)] = output_data
                        .output
                        .unlock_conditions()
                        .expect("output needs to have unlock conditions")
                        .as_ref()
                    {
                        // add nft_id for nft outputs
                        if let Output::Nft(output) = &output_data.output {
                            let nft_id = output.nft_id().or_from_output_id(output_data.output_id);
                            nfts.push(nft_id);
                        }

                        // Add amount
                        total_amount += output_data.output.amount();
                        // Add storage deposit
                        required_storage_deposit += &output_data.output.rent_cost(&rent_structure);
                        // Add native tokens
                        if let Some(native_tokens) = output_data.output.native_tokens() {
                            total_native_tokens.add_native_tokens(native_tokens.clone())?;
                        }
                    } else {
                        // if we have multiple unlock conditions for basic or nft outputs, then we might can't spend the
                        // balance at the moment or in the future

                        let output_can_be_unlocked_now =
                            unlockable_outputs_with_multiple_unlock_conditions.contains(&output_data.output_id);

                        // For outputs that are expired or have a timelock unlock condition, but no expiration unlock
                        // condition and we then can unlock them, then they can never be not available for us anymore
                        // and should be added to the balance
                        if output_can_be_unlocked_now {
                            // check if output can be unlocked always from now on, in that case it should be added to
                            // the total amount
                            let output_can_be_unlocked_now_and_in_future = can_output_be_unlocked_forever_from_now_on(
                                // We use the addresses with unspent outputs, because other addresses of the
                                // account without unspent outputs can't be related to this output
                                &account.addresses_with_unspent_outputs,
                                output_data,
                                local_time,
                            );

                            if output_can_be_unlocked_now_and_in_future {
                                // If output has a StorageDepositReturnUnlockCondition, the amount of it should be
                                // subtracted, because this part needs to be sent back
                                let amount = if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
                                    if let Some(sdr) = unlock_conditions.storage_deposit_return() {
                                        if account_addresses
                                            .iter()
                                            .any(|a| a.address.inner == *sdr.return_address())
                                        {
                                            // sending to ourself, we get the full amount
                                            output_data.output.amount()
                                        } else {
                                            // Sending to someone else
                                            output_data.output.amount() - sdr.amount()
                                        }
                                    } else {
                                        output_data.output.amount()
                                    }
                                } else {
                                    output_data.output.amount()
                                };

                                // add nft_id for nft outputs
                                if let Output::Nft(output) = &output_data.output {
                                    let nft_id = output.nft_id().or_from_output_id(output_data.output_id);
                                    nfts.push(nft_id);
                                }

                                // Add amount
                                total_amount += amount;
                                // Add storage deposit
                                required_storage_deposit += output_data.output.rent_cost(&rent_structure);
                                // Add native tokens
                                if let Some(native_tokens) = output_data.output.native_tokens() {
                                    total_native_tokens.add_native_tokens(native_tokens.clone())?;
                                }
                            } else {
                                // only add outputs that can't be locked now and at any point in the future
                                potentially_locked_outputs.insert(output_data.output_id, true);
                            }
                        } else {
                            // Don't add expired outputs that can't ever be unlocked by us
                            if let Some(expiration) = output_data
                                .output
                                .unlock_conditions()
                                .expect("output needs to have unlock conditions")
                                .expiration()
                            {
                                // Not expired, could get unlockable when it's expired, so we insert it
                                if local_time < expiration.timestamp() {
                                    potentially_locked_outputs.insert(output_data.output_id, false);
                                }
                            } else {
                                potentially_locked_outputs.insert(output_data.output_id, false);
                            }
                        }
                    }
                }
            }
        }

        // for `available` get locked_outputs, sum outputs amount and subtract from total_amount
        log::debug!("[BALANCE] locked outputs: {:#?}", account.locked_outputs);
        let mut locked_amount = 0;
        let mut locked_native_tokens = NativeTokensBuilder::new();

        for locked_output in &account.locked_outputs {
            if let Some(output_data) = account.unspent_outputs.get(locked_output) {
                // Only check outputs that are in this network
                if output_data.network_id == network_id {
                    locked_amount += output_data.output.amount();
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        locked_native_tokens.add_native_tokens(native_tokens.clone())?;
                    }
                }
            }
        }
        log::debug!(
            "[BALANCE] total_amount: {}, locked balance: {}",
            total_amount,
            locked_amount
        );
        if total_amount < locked_amount {
            log::warn!("[BALANCE] total_balance is smaller than the available balance");
            // It can happen that the locked_amount is greater than the available balance if a transaction wasn't
            // confirmed when it got checked during syncing, but shortly after, when the outputs from the address were
            // requested, so we just overwrite the locked_amount
            locked_amount = total_amount;
        };

        let mut native_tokens_balance = Vec::new();

        for native_token in total_native_tokens.finish_vec()? {
            // Check if some amount is currently locked
            let locked_amount = locked_native_tokens.iter().find_map(|(id, amount)| {
                if id == native_token.token_id() {
                    Some(amount)
                } else {
                    None
                }
            });

            native_tokens_balance.push(NativeTokensBalance {
                token_id: *native_token.token_id(),
                total: native_token.amount(),
                available: native_token.amount() - *locked_amount.unwrap_or(&U256::from(0u8)),
            })
        }

        Ok(AccountBalance {
            base_coin: BaseCoinBalance {
                total: total_amount,
                available: total_amount - locked_amount,
            },
            native_tokens: native_tokens_balance,
            required_storage_deposit,
            aliases,
            foundries,
            nfts,
            potentially_locked_outputs,
        })
    }
}

pub(crate) fn add_balances(balances: Vec<AccountBalance>) -> crate::Result<AccountBalance> {
    let mut total_balance: AccountBalance = Default::default();

    for balance in balances {
        total_balance.base_coin.total += balance.base_coin.total;
        total_balance.base_coin.available += balance.base_coin.available;
        total_balance.required_storage_deposit += balance.required_storage_deposit;
        total_balance.nfts.extend(balance.nfts.into_iter());
        total_balance.aliases.extend(balance.aliases.into_iter());
        total_balance.foundries.extend(balance.foundries.into_iter());
        for native_token_balance in &balance.native_tokens {
            if let Some(total_native_token_balance) = total_balance
                .native_tokens
                .iter_mut()
                .find(|n| n.token_id == native_token_balance.token_id)
            {
                total_native_token_balance.total += native_token_balance.total;
                total_native_token_balance.available += native_token_balance.available;
            } else {
                total_balance.native_tokens.push(NativeTokensBalance {
                    token_id: native_token_balance.token_id,
                    total: native_token_balance.total,
                    available: native_token_balance.available,
                })
            }
        }
    }

    Ok(total_balance)
}
