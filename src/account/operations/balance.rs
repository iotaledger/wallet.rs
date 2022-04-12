// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{hash_map::Entry, HashMap};

use iota_client::bee_message::output::{
    unlock_condition::StorageDepositReturnUnlockCondition, ByteCost, Output, UnlockCondition,
};

use crate::account::{handle::AccountHandle, types::AccountBalance, OutputsToCollect};

impl AccountHandle {
    /// Get the AccountBalance
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        log::debug!("[BALANCE] get balance");
        let unlockable_outputs_with_multiple_unlock_conditions = self
            .get_unlockable_outputs_with_additional_unlock_conditions(OutputsToCollect::All)
            .await?;

        let account = self.read().await;

        let network_id = self.client.get_network_id().await?;
        let byte_cost_config = self.client.get_byte_cost_config().await?;
        // todo: use this to determine which outputs can be spent now
        let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;

        let mut total_amount = 0;
        let mut required_storage_deposit = 0;
        let mut total_native_tokens = HashMap::new();
        let mut potential_locked_outputs = HashMap::new();
        let mut aliases = Vec::new();
        let mut foundries = Vec::new();
        let mut nfts = Vec::new();

        for output_data in account.unspent_outputs.values() {
            // Check if output is from the network we're currently connected to
            if output_data.network_id == network_id {
                // Add alias and foundry outputs here because they can't have a [`StorageDepositReturnUnlockCondition`]
                // or time related unlock conditions
                match &output_data.output {
                    Output::Foundry(output) => foundries.push(output.id()),
                    Output::Alias(output) => {
                        let alias_id = output.alias_id().or_from_output_id(output_data.output_id);
                        aliases.push(alias_id);
                    }
                    _ => {}
                }

                // If there is only an [AddressUnlockCondition], then we can spend the output at any time without
                // restrictions
                if output_data
                    .output
                    .unlock_conditions()
                    .expect("no unlock_conditions")
                    .len()
                    == 1
                {
                    // add nft_id for nft outputs
                    if let Output::Nft(output) = &output_data.output {
                        let nft_id = output.nft_id().or_from_output_id(output_data.output_id);
                        nfts.push(nft_id);
                    }

                    // Add amount
                    total_amount += output_data.output.amount();
                    // Add storage deposit
                    required_storage_deposit += &output_data.output.byte_cost(&byte_cost_config);
                    // Add native tokens
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        for native_token in native_tokens.iter() {
                            match total_native_tokens.entry(*native_token.token_id()) {
                                Entry::Vacant(e) => {
                                    e.insert(*native_token.amount());
                                }
                                Entry::Occupied(mut e) => {
                                    *e.get_mut() += *native_token.amount();
                                }
                            }
                        }
                    }
                } else {
                    // if we have multiple unlock conditions for basic or nft outputs, then we might can't spend the
                    // balance at the moment or in the future

                    let output_can_be_unlocked_now =
                        unlockable_outputs_with_multiple_unlock_conditions.contains(&output_data.output_id);
                    if !output_can_be_unlocked_now {
                        potential_locked_outputs.insert(output_data.output_id, false);
                    }

                    // For outputs that are expired or have a timelock unlock condition, but no expiration unlock
                    // condition and we then can unlock them, then they can never be not available for us anymore and
                    // should be added to the balance
                    if output_can_be_unlocked_now {
                        // check if output can be unlocked always from now on, in that case it should be added to
                        // the total amount
                        let output_can_be_unlocked_now_and_in_future =
                            crate::account::operations::helpers::time::can_output_be_unlocked_forever_from_now_on(
                                // We use the addresses with unspent outputs, because other addresses of the
                                // account without unspent outputs can't be related to this output
                                &account.addresses_with_unspent_outputs,
                                output_data,
                                local_time as u32,
                                milestone_index,
                            );

                        if output_can_be_unlocked_now_and_in_future {
                            // If output has a StorageDepositReturnUnlockCondition, the amount of it should be
                            // subtracted, because this part needs to be sent back
                            let amount = if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
                                if let Some(UnlockCondition::StorageDepositReturn(sdr)) =
                                    unlock_conditions.get(StorageDepositReturnUnlockCondition::KIND)
                                {
                                    output_data.output.amount() - sdr.amount()
                                } else {
                                    output_data.output.amount()
                                }
                            } else {
                                output_data.output.amount()
                            };

                            // Add amount
                            total_amount += amount;
                            // Add storage deposit
                            required_storage_deposit += output_data.output.byte_cost(&byte_cost_config);
                            // Add native tokens
                            if let Some(native_tokens) = output_data.output.native_tokens() {
                                for native_token in native_tokens.iter() {
                                    match total_native_tokens.entry(*native_token.token_id()) {
                                        Entry::Vacant(e) => {
                                            e.insert(*native_token.amount());
                                        }
                                        Entry::Occupied(mut e) => {
                                            *e.get_mut() += *native_token.amount();
                                        }
                                    }
                                }
                            }
                        } else {
                            // only add outputs that can't be locked now and at any point in the future
                            potential_locked_outputs.insert(output_data.output_id, true);
                        }
                    } else {
                        potential_locked_outputs.insert(output_data.output_id, false);
                    }
                }
            }
        }

        // for `available` get locked_outputs, sum outputs amount and subtract from total_amount
        log::debug!("[BALANCE] locked outputs: {:#?}", account.locked_outputs);
        let mut locked_amount = 0;

        for locked_output in &account.locked_outputs {
            if let Some(output) = account.unspent_outputs.get(locked_output) {
                // Only check outputs that are in this network
                if output.network_id == network_id {
                    locked_amount += output.amount;
                }
            }
        }
        log::debug!(
            "[BALANCE] total_amount: {}, lockedbalance: {}",
            total_amount,
            locked_amount
        );
        if total_amount < locked_amount {
            log::warn!("[BALANCE] total_balance is smaller than the available balance");
            // It can happen that the locked_amount is greater than the available blance if a transaction wasn't
            // confirmed when it got checked during syncing, but shortly after, when the outputs from the address were
            // requested, so we just overwrite the locked_amount
            locked_amount = total_amount;
        };
        Ok(AccountBalance {
            total: total_amount,
            available: total_amount - locked_amount,
            native_tokens: total_native_tokens,
            required_storage_deposit,
            aliases,
            foundries,
            nfts,
            potential_locked_outputs,
        })
    }
}
