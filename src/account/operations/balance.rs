// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, types::AccountBalance};

use iota_client::bee_message::output::{AliasId, ByteCost, ByteCostConfigBuilder, NftId, Output};

use std::collections::{hash_map::Entry, HashMap};

impl AccountHandle {
    /// Get the AccountBalance
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        log::debug!("[BALANCE] get balance");
        let account = self.read().await;

        let network_id = self.client.get_network_id().await?;
        let rent_structure = self.client.get_rent_structure().await?;
        let byte_cost_config = ByteCostConfigBuilder::new()
            .byte_cost(rent_structure.v_byte_cost)
            .key_factor(rent_structure.v_byte_factor_key)
            .data_factor(rent_structure.v_byte_factor_data)
            .finish();
        // todo: use this to determine which outputs can be spent now
        // let (local_time, milestone_index) = self.get_time_and_milestone_checked().await?;

        let mut total_balance = 0;
        let mut locked_amount = 0;
        let mut required_storage_deposit = 0;
        let mut locked_required_storage_deposit = 0;
        let mut total_native_tokens = HashMap::new();
        let mut locked_native_tokens = HashMap::new();
        let mut aliases = Vec::new();
        let mut foundries = Vec::new();
        let mut nfts = Vec::new();
        let mut locked_nfts = Vec::new();

        for output_data in account.unspent_outputs.values() {
            if output_data.network_id == network_id {
                // If there is only an [AddressUnlockCondition] or [ImmutableAliasAddressUnlockCondition], then we can
                // control the balance
                if output_data
                    .output
                    .unlock_conditions()
                    .expect("no unlock_conditions")
                    .len()
                    == 1
                {
                    // Add amount
                    total_balance += output_data.output.amount();
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
                    // add foundry and nft outputs
                    match &output_data.output {
                        Output::Foundry(output) => foundries.push(output.id()),
                        Output::Nft(output) => {
                            // When the nft is minted, the nft_id contains only `0` bytes and we need to calculate the
                            // output id
                            // todo: replace with `.or_from_output_id(output_data.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                            let nft_id = if output.nft_id().iter().all(|&b| b == 0) {
                                NftId::from(&output_data.output_id)
                            } else {
                                *output.nft_id()
                            };
                            nfts.push(nft_id);
                        }
                        // Alias outputs are ignored here, because they always need two unlock conditions
                        _ => {}
                    }
                } else {
                    // if we have other unlock conditions added for basic or nft outputs, then we might can't spend the
                    // balance at the moment or in the future, because it expired
                    // Add amount
                    locked_amount += output_data.output.amount();
                    // Add storage deposit
                    locked_required_storage_deposit += output_data.output.byte_cost(&byte_cost_config);
                    // Add native tokens
                    if let Some(native_tokens) = output_data.output.native_tokens() {
                        for native_token in native_tokens.iter() {
                            match locked_native_tokens.entry(*native_token.token_id()) {
                                Entry::Vacant(e) => {
                                    e.insert(*native_token.amount());
                                }
                                Entry::Occupied(mut e) => {
                                    *e.get_mut() += *native_token.amount();
                                }
                            }
                        }
                    }
                    // add alias, foundry and nft outputs
                    match &output_data.output {
                        Output::Alias(output) => {
                            // When the nft is minted, the alias_id contains only `0` bytes and we need to calculate the
                            // output id
                            // todo: replace with `.or_from_output_id(output_data.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                            let alias_id = if output.alias_id().iter().all(|&b| b == 0) {
                                AliasId::from(&output_data.output_id)
                            } else {
                                *output.alias_id()
                            };
                            aliases.push(alias_id);
                        }
                        Output::Foundry(output) => foundries.push(output.id()),
                        Output::Nft(output) => {
                            // When the nft is minted, the nft_id contains only `0` bytes and we need to calculate the
                            // output id
                            // todo: replace with `.or_from_output_id(output_data.output_id)` when available in bee: https://github.com/iotaledger/bee/pull/977
                            let nft_id = if output.nft_id().iter().all(|&b| b == 0) {
                                NftId::from(&output_data.output_id)
                            } else {
                                *output.nft_id()
                            };
                            locked_nfts.push(nft_id);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Balance from the outputs and addresses_with_balance should match
        #[cfg(debug_assertions)]
        assert_eq!(
            total_balance,
            account.addresses_with_balance.iter().map(|a| a.amount()).sum::<u64>()
        );

        // for `available` get locked_outputs, sum outputs balance and subtract from total_balance
        log::debug!("[BALANCE] locked outputs: {:#?}", account.locked_outputs);
        let mut locked_balance = 0;

        for locked_output in &account.locked_outputs {
            if let Some(output) = account.unspent_outputs.get(locked_output) {
                // Only check outputs that are in this network
                if output.network_id == network_id {
                    locked_balance += output.amount;
                }
            }
        }
        log::debug!(
            "[BALANCE] total_balance: {}, lockedbalance: {}",
            total_balance,
            locked_balance
        );
        if total_balance < locked_balance {
            log::warn!("[BALANCE] total_balance is smaller than the available balance");
            // It can happen that the locked_balance is greater than the available blance if a transaction wasn't
            // confirmed when it got checked during syncing, but shortly after, when the outputs from the address were
            // requested, so we just overwrite the locked_balance
            locked_balance = total_balance;
        };
        Ok(AccountBalance {
            total: total_balance,
            locked_amount,
            available: total_balance - locked_balance,
            native_tokens: total_native_tokens,
            locked_native_tokens,
            required_storage_deposit,
            locked_required_storage_deposit,
            aliases,
            foundries,
            nfts,
            locked_nfts,
        })
    }
}
