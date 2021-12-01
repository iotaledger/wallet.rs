// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod addresses;
pub mod options;
pub(crate) mod outputs;
pub(crate) mod transactions;
use crate::account::{
    constants::MIN_SYNC_INTERVAL,
    handle::AccountHandle,
    operations::output_consolidation::consolidate_outputs,
    types::{address::AddressWithBalance, InclusionState, OutputData, Transaction},
    AccountBalance,
};
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
use crate::signing::SignerType;
pub use options::SyncOptions;

use iota_client::bee_message::output::OutputId;

use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Syncs an account
pub async fn sync_account(account_handle: &AccountHandle, options: &SyncOptions) -> crate::Result<AccountBalance> {
    log::debug!("[SYNC] start syncing with {:?}", options);
    let syc_start_time = Instant::now();

    // prevent syncing the account multiple times simultaneously
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    let mut last_synced = account_handle.last_synced.lock().await;
    log::debug!("[SYNC] last time synced before {}ms", time_now - *last_synced);
    if time_now - *last_synced < MIN_SYNC_INTERVAL && !options.force_syncing {
        log::debug!(
            "[SYNC] synced within the latest {} ms, only calculating balance",
            MIN_SYNC_INTERVAL
        );
        // calculate the balance because if we created a transaction the amount for the inputs is not available anymore
        return account_handle.balance().await;
    }

    // sync transactions first so we maybe get confirmed outputs in the syncing process later
    // do we want a field in SyncOptions so it can be skipped?
    let (synced_transactions, spent_output_ids) = transactions::sync_transactions(account_handle).await?;

    // we get the balance first because it's a less heavy operation for the nodes
    let addresses_with_balance = addresses::get_addresses_with_balance(account_handle, options).await?;
    log::debug!("[SYNC] found {} addresses_with_balance", addresses_with_balance.len());

    // get outputs only for addresses that have > 0 as balance and add them also the the addresses_with_balance
    let (new_output_ids, addresses_with_balance) =
        addresses::get_address_output_ids(account_handle, options, addresses_with_balance.clone()).await?;

    let output_responses = outputs::get_outputs(account_handle, options, new_output_ids.clone()).await?;
    let outputs = outputs::output_response_to_output_data(account_handle, output_responses).await?;

    // only when actively called or also in the background syncing?
    let signer_type = {
        let account = account_handle.read().await;
        account.signer_type.clone()
    };
    match signer_type {
        #[cfg(feature = "ledger-nano")]
        // don't automatically consoldiate with ledger accounts, because they require approval from the user
        SignerType::LedgerNano => {}
        #[cfg(feature = "ledger-nano-simulator")]
        SignerType::LedgerNanoSimulator => {}
        _ => {
            consolidate_outputs(account_handle).await?;
        }
    };

    // add a field to the sync options to also sync incoming transactions?

    // update account with balances, output ids, outputs
    update_account(
        account_handle,
        addresses_with_balance,
        outputs,
        synced_transactions,
        spent_output_ids,
        options,
    )
    .await?;
    // store account with storage feature

    let account_balance = account_handle.balance().await?;
    // update last_synced mutex
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    *last_synced = time_now;
    log::debug!("[SYNC] finished syncing in {:.2?}", syc_start_time.elapsed());
    Ok(account_balance)
}

/// Update account with newly synced data
async fn update_account(
    account_handle: &AccountHandle,
    addresses_with_balance: Vec<AddressWithBalance>,
    outputs: Vec<OutputData>,
    synced_transactions: Vec<Transaction>,
    spent_output_ids: Vec<OutputId>,
    options: &SyncOptions,
) -> crate::Result<()> {
    let mut account = account_handle.write().await;
    // update used field of the addresses
    for address in addresses_with_balance.iter() {
        if address.internal {
            let position = account
                .internal_addresses
                .binary_search_by_key(&(address.key_index, address.internal), |a| (a.key_index, a.internal))
                .map_err(|e| crate::Error::InputAddressNotFound)?;
            account.internal_addresses[position].used = true;
        } else {
            let position = account
                .public_addresses
                .binary_search_by_key(&(address.key_index, address.internal), |a| (a.key_index, a.internal))
                .map_err(|e| crate::Error::InputAddressNotFound)?;
            account.public_addresses[position].used = true;
        }
    }
    // get all addresses with balance that we didn't sync because their index is below the address_start_index of the
    // options
    account.addresses_with_balance = account
        .addresses_with_balance
        .iter()
        .filter(|a| a.key_index < options.address_start_index)
        .cloned()
        .collect();
    // then add all synced addresses with balance
    account.addresses_with_balance.extend(addresses_with_balance);

    for output in outputs {
        account.outputs.insert(output.output_id, output.clone());
        if !output.is_spent {
            account.unspent_outputs.insert(output.output_id, output);
        }
    }

    for transaction in synced_transactions {
        match transaction.inclusion_state {
            InclusionState::Confirmed | InclusionState::Conflicting => {
                account.pending_transactions.remove(&transaction.payload.id());
            }
            _ => {}
        }
        account.transactions.insert(transaction.payload.id(), transaction);
    }

    for spent_output_id in spent_output_ids {
        if let Some(output) = account.outputs.get_mut(&spent_output_id) {
            output.is_spent = true;
        }
        if let Some(output) = account.unspent_outputs.get_mut(&spent_output_id) {
            output.is_spent = true;
        }
        account.locked_outputs.remove(&spent_output_id);
        account.unspent_outputs.remove(&spent_output_id);
        log::debug!("[SYNC] Unlocked {}", spent_output_id);
    }
    #[cfg(feature = "storage")]
    log::debug!("[SYNC] storing account {}", account.index());
    crate::storage::manager::get()
        .await?
        .lock()
        .await
        .save_account(&account)
        .await?;
    // println!("{:#?}", account);
    Ok(())
}

// have an own function to sync spent outputs? (only for history reasons, not important now)
// async fn get_spent_outputs(
//     account_handle: &AccountHandle,
//     options: &SyncOptions,
// ) -> crate::Result<Vec<Output>> {
//     Ok(vec![])
// }
