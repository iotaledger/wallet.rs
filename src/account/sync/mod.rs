// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountHandle},
    account_manager::{AccountOptions, AccountStore},
    address::{Address, AddressBuilder, AddressOutput, AddressWrapper, OutputKind},
    client::ClientOptions,
    event::{
        emit_balance_change, emit_confirmation_state_change, emit_transaction_event, AddressData, BalanceChange,
        PreparedTransactionData, TransactionEventType, TransactionIO, TransferProgressType,
    },
    message::{Message, MessageType, RemainderValueStrategy, Transfer},
    signing::{GenerateAddressMetadata, SignMessageMetadata, SignerType},
};

use getset::Getters;
use iota_client::{
    api::finish_pow,
    bee_message::{
        constants::INPUT_OUTPUT_COUNT_MAX,
        prelude::{
            Essence, Input, Message as IotaMessage, MessageId, Output, OutputId, Payload, RegularEssence,
            SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput, TransactionPayload, UnlockBlocks,
            UtxoInput,
        },
    },
    common::packable::Packable,
    AddressOutputsOptions, Client,
};
use serde::Serialize;
use tokio::sync::MutexGuard;

use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

mod input_selection;

// https://github.com/GalRogozinski/protocol-rfcs/blob/dust/text/0032-dust-protection/0032-dust-protection.md
const MAX_ALLOWED_DUST_OUTPUTS: i64 = 100;
const DUST_DIVISOR: i64 = 100_000;
const DUST_ALLOWANCE_VALUE: u64 = 1_000_000;
const DEFAULT_GAP_LIMIT: usize = 10;
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
const DEFAULT_LEDGER_GAP_LIMIT: usize = 10;
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
const LEDGER_MAX_IN_OUTPUTS: usize = 17;
const SYNC_CHUNK_SIZE: usize = 500;

#[derive(Debug, Clone)]
pub(crate) struct SyncedMessage {
    pub(crate) id: MessageId,
    pub(crate) inner: IotaMessage,
}

async fn get_address_outputs(
    address: String,
    client: &Client,
    fetch_spent_outputs: bool,
) -> crate::Result<Vec<(UtxoInput, bool)>> {
    let address_outputs = client
        .get_address()
        .outputs(
            &address,
            AddressOutputsOptions {
                include_spent: fetch_spent_outputs,
                ..Default::default()
            },
        )
        .await?
        .to_vec();

    // fetch only the unspent outputs so we can diff it
    if fetch_spent_outputs {
        let unspent_address_outputs = client
            .get_address()
            .outputs(
                &address,
                AddressOutputsOptions {
                    include_spent: false,
                    ..Default::default()
                },
            )
            .await?;
        let spent_address_outputs: Vec<(UtxoInput, bool)> = address_outputs
            .into_iter()
            .filter(|o| !unspent_address_outputs.contains(o))
            .map(|o| (o, true))
            .collect();

        let mut outputs: Vec<(UtxoInput, bool)> = unspent_address_outputs.iter().map(|o| (o.clone(), false)).collect();
        outputs.extend(spent_address_outputs);
        Ok(outputs)
    } else {
        Ok(address_outputs.into_iter().map(|output| (output, false)).collect())
    }
}

async fn get_message(client: &Client, message_id: &MessageId) -> crate::Result<Option<IotaMessage>> {
    match client.get_message().data(message_id).await {
        Ok(message) => Ok(Some(message)),
        Err(iota_client::Error::ResponseError(status_code, _)) if status_code == 404 => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub(crate) async fn sync_address(
    account_messages: Vec<(MessageId, Option<bool>)>,
    client_options: &ClientOptions,
    outputs: &mut HashMap<OutputId, AddressOutput>,
    iota_address: AddressWrapper,
    bech32_hrp: String,
    options: AccountOptions,
) -> crate::Result<Vec<SyncedMessage>> {
    let client_guard = crate::client::get_client(client_options).await?;
    let client = client_guard.read().await;

    let address_outputs = get_address_outputs(iota_address.to_bech32(), &client, options.sync_spent_outputs).await?;
    drop(client);

    let mut found_messages = vec![];

    log::debug!(
        "[SYNC] syncing address {}, got {} outputs",
        iota_address.to_bech32(),
        address_outputs.len(),
    );

    for (output_id, output) in outputs.iter_mut() {
        // if we previously had an output that wasn't returned by the node, mark it as spent
        if !address_outputs
            .iter()
            .any(|(utxo_input, _)| utxo_input.output_id() == output_id)
        {
            output.set_is_spent(true);
        }
    }

    let mut tasks = Vec::new();
    for (utxo_input, is_spent) in address_outputs.iter() {
        let utxo_input = utxo_input.clone();
        // if we already have the output we don't need to get the info from the node
        if let Some(existing_output) = outputs.get_mut(utxo_input.output_id()) {
            existing_output.set_is_spent(*is_spent);
            continue;
        }

        let client_guard = client_guard.clone();
        let bech32_hrp = bech32_hrp.clone();
        let spent = *is_spent;
        let account_messages = account_messages.clone();
        tasks.push(async move {
            tokio::spawn(async move {
                let client = client_guard.read().await;
                let output = match client.get_output(&utxo_input).await {
                    Ok(output) => output,
                    Err(err) => {
                        if spent {
                            return Err(crate::Error::SpentOutputNotFound);
                        } else {
                            return Err(err.into());
                        }
                    }
                };
                let found_output = AddressOutput::from_output_response(output, bech32_hrp.to_string())?;
                let message_id = *found_output.message_id();

                // if we already have the message stored
                // and the confirmation state is known
                // we skip the `get_message` call
                if account_messages
                    .iter()
                    .any(|(id, confirmed)| id == &message_id && confirmed.is_some())
                {
                    return crate::Result::Ok((found_output, None));
                }

                if let Some(message) = get_message(&client, &message_id).await? {
                    return Ok((
                        found_output,
                        Some(SyncedMessage {
                            id: message_id,
                            inner: message,
                        }),
                    ));
                }

                Ok((found_output, None))
            })
            .await
        });
    }

    for res in futures::future::try_join_all(tasks).await? {
        // Don't return an error if we didn't got a spent output
        match res {
            Ok((found_output, found_message)) => {
                outputs.insert(found_output.id()?, found_output);
                if let Some(m) = found_message {
                    found_messages.push(m);
                }
            }
            Err(e) => match e {
                crate::Error::SpentOutputNotFound => {
                    log::debug!("[SYNC] output not found in syncing address")
                }
                _ => return Err(e),
            },
        }
    }

    crate::Result::Ok(found_messages)
}

// Gets an address for the sync process.
// If the account already has the address with the given index + internal flag, we'll use it
// otherwise we'll generate a new one.
async fn get_address_for_sync(
    account: &Account,
    bech32_hrp: String,
    index: usize,
    internal: bool,
) -> crate::Result<Option<AddressWrapper>> {
    if let Some(address) = account
        .addresses()
        .iter()
        .find(|a| *a.key_index() == index && *a.internal() == internal)
    {
        Ok(Some(address.address().clone()))
    } else {
        // if stronghold is locked, we skip address generation
        #[cfg(feature = "stronghold")]
        {
            if account.signer_type() == &crate::signing::SignerType::Stronghold
                && crate::stronghold::get_status(
                    &crate::signing::stronghold::stronghold_path(account.storage_path()).await?,
                )
                .await
                .snapshot
                    == crate::stronghold::SnapshotStatus::Locked
            {
                return Ok(None);
            }
        }
        let generated_address = crate::address::get_iota_address(
            account,
            index,
            internal,
            bech32_hrp,
            GenerateAddressMetadata {
                syncing: true,
                network: account.network(),
            },
        )
        .await?;
        Ok(Some(generated_address))
    }
}

async fn sync_address_list(
    addresses: Vec<Address>,
    account_messages: Vec<(MessageId, Option<bool>)>,
    options: AccountOptions,
    client_options: ClientOptions,
    return_all_addresses: bool,
) -> crate::Result<(Vec<Address>, Vec<SyncedMessage>)> {
    let mut found_addresses = Vec::new();
    let mut found_messages = Vec::new();

    log::debug!("[SYNC] address_list length: {}", addresses.len());

    // We split the addresses into chunks so we don't get timeouts if we have thousands
    for addresses_chunk in addresses
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[Address]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let mut address = address.clone();
            let account_messages = account_messages.clone();
            let mut outputs = address.outputs().clone();
            let client_options = client_options.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let messages = sync_address(
                        account_messages,
                        &client_options,
                        &mut outputs,
                        address.address().clone(),
                        address.address().bech32_hrp.clone(),
                        options,
                    )
                    .await?;
                    address.set_outputs(outputs);
                    crate::Result::Ok((messages, address))
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        let mut inserted_remainder_address = false;
        for res in results {
            let (messages, address) = res?;
            if !address.outputs().is_empty() || return_all_addresses {
                found_addresses.push(address);
            } else if !inserted_remainder_address {
                // We want to insert one unused address to have an unused remainder address
                found_addresses.push(address);
                inserted_remainder_address = true;
            }
            found_messages.extend(messages);
        }
    }

    Ok((found_addresses, found_messages))
}

/// Syncs addresses with the tangle.
/// The method ensures that the wallet local state has all used addresses plus an unused address.
///
/// To sync addresses for an account from scratch, `address_index` = 0 and `gap_limit` = 10 should be provided.
/// To sync addresses from the latest address, `address_index` = latest address index and `gap_limit` = 1 should be
/// provided.
///
/// # Arguments
///
/// * `address_index` The address index.
/// * `gap_limit` Number of addresses indexes that are generated.
///
/// # Return value
///
/// Returns a (addresses, messages) tuples representing the address history up to latest unused address,
/// and the messages associated with the addresses.
async fn sync_addresses(
    account_handle: &AccountHandle,
    internal: bool,
    address_index: usize,
    gap_limit: usize,
    options: AccountOptions,
    return_all_addresses: bool,
) -> crate::Result<(Vec<Address>, Vec<SyncedMessage>)> {
    log::debug!("[SYNC] sync_addresses internal: {}", internal);
    let mut address_index = address_index;

    let mut generated_addresses = vec![];
    let mut found_messages = vec![];

    let account = account_handle.read().await.clone();
    let bech32_hrp = account.bech32_hrp().clone();
    drop(account);

    loop {
        let mut address_generation_locked = false;
        let mut generated_iota_addresses = vec![]; // collection of (address_index, address) pairs
        for i in address_index..(address_index + gap_limit) {
            // generate addresses
            let account = account_handle.read().await.clone();
            if let Some(address) = get_address_for_sync(&account, bech32_hrp.to_string(), i, internal).await? {
                generated_iota_addresses.push((i, address));
            } else {
                address_generation_locked = true;
                break;
            }
            drop(account);
        }

        let mut curr_generated_addresses = vec![];
        let mut curr_found_messages = vec![];

        let account = account_handle.read().await.clone();
        let account_addresses: Vec<(AddressWrapper, HashMap<OutputId, AddressOutput>)> = account
            .addresses()
            .iter()
            .map(|a| (a.address().clone(), a.outputs().clone()))
            .collect();
        let account_messages: Vec<(MessageId, Option<bool>)> = account
            .with_messages(|messages| messages.iter().map(|m| (m.key, m.confirmed)).collect())
            .await;
        let client_options = account.client_options().clone();
        drop(account);

        let mut addresses_to_sync = Vec::new();
        for (iota_address_index, iota_address) in generated_iota_addresses.to_vec() {
            let outputs = account_addresses
                .iter()
                .find(|(a, _)| a == &iota_address)
                .map(|(_, outputs)| outputs.clone())
                .unwrap_or_default();
            let address = AddressBuilder::new()
                .address(iota_address.clone())
                .key_index(iota_address_index)
                .outputs(outputs.values().cloned().collect())
                .internal(internal)
                .build()?;
            addresses_to_sync.push(address);
        }

        let (found_addresses_, found_messages_) = sync_address_list(
            addresses_to_sync,
            account_messages,
            options,
            client_options.clone(),
            return_all_addresses,
        )
        .await?;
        curr_generated_addresses.extend(found_addresses_);
        curr_found_messages.extend(found_messages_);

        address_index += gap_limit;

        let is_empty = curr_found_messages.is_empty()
            && curr_generated_addresses
                .iter()
                .all(|address| address.outputs().is_empty());

        found_messages.extend(curr_found_messages.into_iter());
        generated_addresses.extend(curr_generated_addresses.into_iter());

        if is_empty {
            log::debug!(
                "[SYNC] finishing address syncing because the current messages list and address list are empty"
            );
            break;
        }

        if address_generation_locked {
            log::debug!("[SYNC] finishing address syncing because stronghold is locked");
            break;
        }
    }

    Ok((generated_addresses, found_messages))
}

/// Syncs messages with the tangle.
/// The method should ensures that the wallet local state has messages associated with the address history.
async fn sync_messages(
    account_handle: &AccountHandle,
    skip_addresses: &[Address],
    options: AccountOptions,
    skip_change_addresses: bool,
    change_addresses_to_sync: HashSet<AddressWrapper>,
) -> crate::Result<(Vec<Address>, Vec<SyncedMessage>)> {
    log::debug!("[SYNC] sync_messages");
    let mut messages = vec![];

    let account = account_handle.read().await.clone();
    let client_options = account.client_options().clone();

    let messages_with_known_confirmation: Vec<MessageId> = account
        .with_messages(|messages| {
            messages
                .iter()
                .filter(|m| m.confirmed.is_some())
                .map(|m| m.key)
                .collect()
        })
        .await;

    let mut addresses = Vec::new();

    let client = crate::client::get_client(&client_options).await?;

    log::debug!("[SYNC] sync_messages for {} addresses", account.addresses().len());

    // We split the addresses into chunks so we don't get timeouts if we have thousands
    let account_addresses = account.addresses().clone();
    drop(account);
    for addresses_chunk in account_addresses
        .to_vec()
        .chunks(SYNC_CHUNK_SIZE)
        .map(|x: &[Address]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for address in addresses_chunk {
            let mut address = address.clone();
            if skip_addresses.contains(&address)
                || (*address.internal()
                    && skip_change_addresses
                    && !change_addresses_to_sync.contains(address.address()))
            {
                continue;
            }
            let client = client.clone();
            let messages_with_known_confirmation = messages_with_known_confirmation.clone();
            let mut outputs = address.outputs.clone();

            tasks.push(async move {
                    tokio::spawn(async move {
                    let client = client.read().await;
                    // get address balance first, because it's faster than the outputs endpoint
                    let address_balance = client.get_address().balance(&address.address().to_bech32()).await?;
                    let unspent_output_amount = address
                        .outputs()
                        .iter()
                        .map(|(_, o)| o.is_spent)
                        .filter(|is_spent| !is_spent)
                        .count();

                    log::debug!(
                        "[SYNC] syncing messages and outputs for address {} index: {}, got balance: {}, known unspent outputs: {}",
                        address.address().to_bech32(),
                        address.key_index(),
                        address_balance.balance,
                        unspent_output_amount
                    );
                    // return early if the address has no balance and we don't have any unspent outputs
                    if address_balance.balance == 0 && unspent_output_amount == 0 {
                        return crate::Result::Ok((address, vec![]));
                    }

                    let address_outputs =
                        get_address_outputs(address.address().to_bech32(), &client, options.sync_spent_outputs).await?;

                    for (output_id, output) in outputs.iter_mut() {
                        // if we previously had an output that wasn't returned by the node, mark it as spent
                        if !address_outputs
                            .iter()
                            .any(|(utxo_input, _)| utxo_input.output_id() == output_id)
                        {
                            output.set_is_spent(true);
                        }
                    }

                    log::debug!(
                        "[SYNC] syncing messages and outputs for address {}, got {} outputs",
                        address.address().to_bech32(),
                        address_outputs.len(),
                    );

                    let mut messages = vec![];
                    for (utxo_input, is_spent) in address_outputs.iter() {
                        let output = match address.outputs().get(utxo_input.output_id()) {
                            // if we already have the output we don't need to get the info from the node
                            Some(output) => {
                                let mut output = output.clone();
                                output.set_is_spent(*is_spent);
                                output
                            }
                            None => {
                                // if the output got spent and we didn't get it from the node we ignore it and don't return an error
                                if *is_spent {
                                    match client.get_output(utxo_input).await{
                                        Ok(output) => {
                                            AddressOutput::from_output_response(output, address.address().bech32_hrp().to_string())?
                                        }
                                        Err(_) =>{
                                            log::debug!(
                                                "[SYNC] couldn't get spent output: {}",
                                                utxo_input.output_id().transaction_id().to_string(),
                                            );
                                            continue;
                                        }
                                    }
                                } else {
                                    let output = client.get_output(utxo_input).await?;
                                    AddressOutput::from_output_response(output, address.address().bech32_hrp().to_string())?
                                }
                            }
                        };

                        let output_message_id = *output.message_id();
                        outputs.insert(output.id()?, output);

                        // if we already have the message stored
                        // and the confirmation state is known
                        // we skip the `get_message` call
                        if messages_with_known_confirmation.contains(&output_message_id) {
                            continue;
                        }

                        if let Some(message) = get_message(&client, &output_message_id).await? {
                            messages.push(SyncedMessage {
                                id: output_message_id,
                                inner: message,
                            });
                        }
                    }

                    address.set_outputs(outputs);

                    crate::Result::Ok((address, messages))
                })
                .await
            });
        }
        for res in futures::future::try_join_all(tasks).await? {
            let (address, found_messages) = res?;
            if !address.outputs().is_empty() {
                addresses.push(address);
            }
            messages.extend(found_messages);
        }
    }

    Ok((addresses, messages))
}

#[allow(clippy::too_many_arguments)]
async fn perform_sync(
    account_handle: AccountHandle,
    address_index: usize,
    gap_limit: usize,
    skip_change_addresses: bool,
    change_addresses_to_sync: HashSet<AddressWrapper>,
    steps: &[AccountSynchronizeStep],
    options: AccountOptions,
    return_all_addresses: bool,
) -> crate::Result<SyncedAccountData> {
    log::debug!(
        "[SYNC] perform_sync: syncing account {} with address_index = {}, gap_limit = {}",
        account_handle.read().await.index(),
        address_index,
        gap_limit
    );
    let (mut found_addresses, found_messages) = if let Some(index) = steps
        .iter()
        .position(|s| matches!(s, AccountSynchronizeStep::SyncAddresses(_)))
    {
        if let AccountSynchronizeStep::SyncAddresses(addresses) = &steps[index] {
            if let Some(addresses) = addresses {
                log::debug!(
                    "[SYNC] syncing specific addresses: {:?}",
                    addresses.iter().map(|a| a.to_bech32()).collect::<Vec<String>>()
                );
                let account = account_handle.read().await.clone();
                let account_messages: Vec<(MessageId, Option<bool>)> = account
                    .with_messages(|messages| messages.iter().map(|m| (m.key, m.confirmed)).collect())
                    .await;
                let mut addresses_to_sync = Vec::new();
                for address in account.addresses() {
                    if !addresses.contains(address.address()) {
                        continue;
                    }
                    let address = AddressBuilder::new()
                        .address(address.address().clone())
                        .key_index(*address.key_index())
                        .outputs(Vec::new())
                        .internal(*address.internal())
                        .build()?;
                    addresses_to_sync.push(address);
                }
                drop(account);
                sync_address_list(
                    addresses_to_sync,
                    account_messages,
                    options,
                    account_handle.read().await.clone().client_options().clone(),
                    return_all_addresses,
                )
                .await?
            } else {
                let (found_public_addresses, mut messages) = sync_addresses(
                    &account_handle,
                    false,
                    address_index,
                    gap_limit,
                    options,
                    return_all_addresses,
                )
                .await?;
                let (found_change_addresses, synced_messages) = sync_addresses(
                    &account_handle,
                    true,
                    address_index,
                    gap_limit,
                    options,
                    return_all_addresses,
                )
                .await?;
                let mut found_addresses = found_public_addresses;
                found_addresses.extend(found_change_addresses);
                messages.extend(synced_messages);
                (found_addresses, messages)
            }
        } else {
            unreachable!()
        }
    } else {
        (Vec::new(), Vec::new())
    };

    let account = account_handle.read().await.clone();
    let mut new_messages = vec![];
    for found_message in found_messages {
        let message_exists = account
            .with_messages(|messages| messages.iter().any(|message| message.key == found_message.id))
            .await;
        if !message_exists {
            new_messages.push(found_message);
        }
    }

    if steps.contains(&AccountSynchronizeStep::SyncMessages) {
        let (synced_addresses, synced_messages) = sync_messages(
            &account_handle,
            &found_addresses,
            options,
            skip_change_addresses,
            change_addresses_to_sync,
        )
        .await?;
        found_addresses.extend(synced_addresses);
        new_messages.extend(synced_messages.into_iter());
    }
    log::debug!("[SYNC] FOUND {:?}", found_addresses);

    // we have two address spaces so we find change & public addresses to save separately
    let mut addresses_to_save = find_addresses_to_save(
        &account,
        found_addresses.iter().filter(|a| !a.internal()).cloned().collect(),
    );
    // Add first public address if there is none, required for account discovery because we always need a public address
    // in an account
    if account.addresses().is_empty() && addresses_to_save.is_empty() && return_all_addresses {
        addresses_to_save.extend(found_addresses.iter().find(|a| !a.internal()).cloned());
    }
    addresses_to_save.extend(find_addresses_to_save(
        &account,
        found_addresses.iter().filter(|a| *a.internal()).cloned().collect(),
    ));

    let new_addresses = addresses_to_save.clone();
    let max_new_public_index = new_addresses
        .iter()
        .filter(|a| !a.internal())
        .max_by_key(|a| a.key_index())
        .map(|a| a.key_index())
        .unwrap_or(&0);
    let max_new_internal_index = new_addresses
        .iter()
        .filter(|a| *a.internal())
        .max_by_key(|a| a.key_index())
        .map(|a| a.key_index())
        .unwrap_or(&0);

    let public_addresses = account.addresses.iter().filter(|a| !a.internal());
    let internal_addresses = account.addresses.iter().filter(|a| *a.internal());
    let latest_public_address_index = public_addresses
        .clone()
        .max_by_key(|a| a.key_index())
        .map(|a| a.key_index())
        .unwrap_or(&0);
    let latest_internal_address_index = internal_addresses
        .clone()
        .max_by_key(|a| a.key_index())
        .map(|a| a.key_index())
        .unwrap_or(&0);

    let bech32_hrp = match account.addresses().first() {
        Some(address) => address.address().bech32_hrp().to_string(),
        None => {
            crate::client::get_client(account.client_options())
                .await?
                .read()
                .await
                .get_network_info()
                .await?
                .bech32_hrp
        }
    };
    // generate all missing addresses
    if !addresses_to_save.is_empty() {
        log::debug!("[SYNC] check for missing addresses");

        // generate missing public addresses
        for key_index in *latest_public_address_index..*max_new_public_index {
            if !account
                .addresses()
                .iter()
                .any(|a| a.key_index() == &key_index && !a.internal())
                && !addresses_to_save
                    .clone()
                    .iter()
                    .any(|a| a.key_index() == &key_index && !a.internal())
            {
                // generate address, ignore errors because Stronghold could be locked or a ledger not connected and we
                // don't want to require an unlock for syncing
                if let Ok(iota_address) = crate::address::get_iota_address(
                    &account,
                    key_index,
                    false,
                    bech32_hrp.clone(),
                    GenerateAddressMetadata {
                        syncing: true,
                        network: account.network(),
                    },
                )
                .await
                {
                    log::debug!(
                        "[SYNC] generated missing public address {} at index {}",
                        iota_address.to_bech32(),
                        key_index
                    );
                    let address = Address {
                        address: iota_address,
                        key_index,
                        internal: false,
                        outputs: Default::default(),
                    };
                    addresses_to_save.push(address);
                };
            }
        }
        // generate missing internal addresses
        for key_index in *latest_internal_address_index..*max_new_internal_index {
            if !account
                .addresses()
                .iter()
                .any(|a| a.key_index() == &key_index && *a.internal())
                && !addresses_to_save
                    .clone()
                    .iter()
                    .any(|a| a.key_index() == &key_index && *a.internal())
            {
                // generate address, ignore errors because Stronghold could be locked or a ledger not connected and we
                // don't want to require an unlock for syncing
                if let Ok(iota_address) = crate::address::get_iota_address(
                    &account,
                    key_index,
                    true,
                    bech32_hrp.clone(),
                    GenerateAddressMetadata {
                        syncing: true,
                        network: account.network(),
                    },
                )
                .await
                {
                    log::debug!(
                        "[SYNC] generated missing internal address {} at index {}",
                        iota_address.to_bech32(),
                        key_index
                    );
                    let address = Address {
                        address: iota_address,
                        key_index,
                        internal: true,
                        outputs: Default::default(),
                    };
                    addresses_to_save.push(address);
                };
            }
        }
    }

    let is_latest_public_address_empty = if latest_public_address_index > max_new_public_index {
        public_addresses
            .max_by_key(|a| a.key_index())
            .map(|a| a.outputs().is_empty())
            .unwrap_or(false)
    } else {
        addresses_to_save
            .iter()
            .filter(|a| !a.internal())
            .max_by_key(|a| a.key_index())
            .map(|a| a.outputs.len())
            .unwrap_or(0)
            == 0
    };
    let is_latest_internal_address_empty = if latest_internal_address_index > max_new_internal_index {
        internal_addresses
            .max_by_key(|a| a.key_index())
            .map(|a| a.outputs().is_empty())
            .unwrap_or(true)
    } else {
        addresses_to_save
            .iter()
            .filter(|a| *a.internal())
            .max_by_key(|a| a.key_index())
            .map(|a| a.outputs.len())
            .unwrap_or(0)
            == 0
    };

    if !is_latest_public_address_empty {
        let latest_index = std::cmp::max(latest_public_address_index, max_new_public_index);
        // generate address, ignore errors because Stronghold could be locked or a ledger not connected and we don't
        // want to require an unlock for syncing
        if let Ok(iota_address) = crate::address::get_iota_address(
            &account,
            latest_index + 1,
            false,
            bech32_hrp.clone(),
            GenerateAddressMetadata {
                syncing: true,
                network: account.network(),
            },
        )
        .await
        {
            log::debug!(
                "[SYNC] generated new unused public address {} at index {}",
                iota_address.to_bech32(),
                latest_index + 1
            );
            let address = Address {
                address: iota_address,
                key_index: latest_index + 1,
                internal: false,
                outputs: Default::default(),
            };
            addresses_to_save.push(address);
        };
    }

    if !is_latest_internal_address_empty {
        let latest_index = std::cmp::max(latest_internal_address_index, max_new_internal_index);
        if let Ok(iota_address) = crate::address::get_iota_address(
            &account,
            latest_index + 1,
            false,
            bech32_hrp.clone(),
            GenerateAddressMetadata {
                syncing: true,
                network: account.network(),
            },
        )
        .await
        {
            log::debug!(
                "[SYNC] generated new unused internal address {} at index {}",
                iota_address.to_bech32(),
                latest_index + 1
            );
            let address = Address {
                address: iota_address,
                key_index: latest_index + 1,
                internal: false,
                outputs: Default::default(),
            };
            addresses_to_save.push(address);
        };
    }

    addresses_to_save.sort_unstable_by_key(|a| *a.key_index());
    addresses_to_save.sort_unstable_by_key(|a| *a.internal());
    addresses_to_save.dedup();

    log::debug!("[SYNC] perform_sync finished");
    Ok(SyncedAccountData {
        messages: new_messages,
        addresses: addresses_to_save,
    })
}

fn find_addresses_to_save(account: &Account, found_addresses: Vec<Address>) -> Vec<Address> {
    let mut addresses_to_save = vec![];
    let mut ignored_addresses = vec![];
    let mut found_addresses = found_addresses;
    found_addresses.sort_unstable_by_key(|a| *a.key_index());
    for found_address in found_addresses.into_iter() {
        let address_is_unused = found_address.outputs().is_empty();

        // if the address was updated, we need to save it
        if let Some(existing_address) = account
            .addresses()
            .iter()
            .find(|a| a.address() == found_address.address())
        {
            if existing_address.outputs() != found_address.outputs() {
                addresses_to_save.push(found_address);
                continue;
            }
        }
        // subsequent unused address found; add it to the ignored addresses list
        if address_is_unused {
            ignored_addresses.push(found_address);
        }
        // used address found after finding unused addresses; we'll save all the previous ignored address and this
        // one aswell
        else {
            addresses_to_save.extend(ignored_addresses.into_iter());
            ignored_addresses = vec![];
            addresses_to_save.push(found_address);
        }
    }

    addresses_to_save
}

#[derive(Clone, PartialEq)]
pub(crate) enum AccountSynchronizeStep {
    SyncAddresses(Option<Vec<AddressWrapper>>),
    SyncMessages,
}

#[derive(Debug, Clone)]
pub(crate) struct BalanceChangeEventData {
    pub(crate) address: AddressWrapper,
    pub(crate) balance_change: BalanceChange,
    pub(crate) message_id: Option<MessageId>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConfirmationChangeEventData {
    pub(crate) message: Message,
    pub(crate) confirmed: bool,
}

/// Account sync helper.
pub struct AccountSynchronizer {
    account_handle: AccountHandle,
    address_index: usize,
    gap_limit: usize,
    skip_persistence: bool,
    skip_change_addresses: bool,
    steps: Vec<AccountSynchronizeStep>,
}

#[derive(Debug)]
pub(crate) struct SyncedAccountData {
    pub(crate) messages: Vec<SyncedMessage>,
    pub(crate) addresses: Vec<Address>,
}

impl SyncedAccountData {
    pub(crate) async fn parse_messages(
        &self,
        accounts: AccountStore,
        account: &Account,
    ) -> crate::Result<Vec<Message>> {
        let mut tasks = Vec::new();
        for new_message in self.messages.to_vec() {
            let client_options = account.client_options().clone();
            let account_id = account.id().to_string();
            let account_addresses = account.addresses().to_vec();
            let accounts = accounts.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    Message::from_iota_message(
                        new_message.id,
                        new_message.inner,
                        accounts,
                        &account_id,
                        &account_addresses,
                        &client_options,
                    )
                    .with_confirmed(Some(true))
                    .finish()
                    .await
                })
                .await
            });
        }
        let mut parsed_messages = Vec::new();
        for message in futures::future::try_join_all(tasks).await? {
            parsed_messages.push(message?);
        }
        Ok(parsed_messages)
    }
}

fn get_balance_change_events(
    old_balance: u64,
    new_balance: u64,
    address: AddressWrapper,
    account_options: AccountOptions,
    before_sync_outputs: HashMap<OutputId, AddressOutput>,
    outputs: &HashMap<OutputId, AddressOutput>,
) -> Vec<BalanceChangeEventData> {
    let mut balance_change_events = Vec::new();
    let mut output_change_balance = 0i64;
    // we use this flag in case the new balance is 0
    let mut emitted_event = false;
    // check new and updated outputs to find message ids
    // note that this is unreliable if we're not syncing spent outputs,
    // since not all information are collected.
    if account_options.sync_spent_outputs {
        for (output_id, output) in outputs {
            if !before_sync_outputs.contains_key(output_id) {
                let balance_change = if output.is_spent {
                    BalanceChange::spent(output.amount)
                } else {
                    BalanceChange::received(output.amount)
                };
                if output.is_spent {
                    output_change_balance -= output.amount as i64;
                } else {
                    output_change_balance += output.amount as i64;
                }
                log::info!("[SYNC] balance change on {} {:?}", address.to_bech32(), balance_change);
                balance_change_events.push(BalanceChangeEventData {
                    address: address.clone(),
                    balance_change,
                    message_id: Some(output.message_id),
                });
                emitted_event = true;
            }
        }
    }

    // we can't guarantee we picked up all output changes since querying spent outputs is
    // optional and the node might prune it so we handle it here; if not all balance change has
    // been emitted, we emit the remainder value with `None` as
    // message_id
    let balance_change = new_balance as i64 - old_balance as i64;
    if !emitted_event || output_change_balance != balance_change {
        let change = new_balance as i64 - old_balance as i64 - output_change_balance;
        let balance_change = if change > 0 {
            BalanceChange::received(change as u64)
        } else {
            BalanceChange::spent(change.abs() as u64)
        };
        log::info!(
            "[SYNC] remaining balance change on {} {:?}",
            address.to_bech32(),
            balance_change
        );
        balance_change_events.push(BalanceChangeEventData {
            address,
            balance_change,
            message_id: None,
        });
    }
    balance_change_events
}

impl AccountSynchronizer {
    /// Initialises a new instance of the sync helper.
    pub(super) async fn new(account_handle: AccountHandle) -> Self {
        let latest_address_index = *account_handle.read().await.latest_address().key_index();
        let default_gap_limit = match account_handle.read().await.signer_type() {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => DEFAULT_LEDGER_GAP_LIMIT,
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => DEFAULT_LEDGER_GAP_LIMIT,
            _ => DEFAULT_GAP_LIMIT,
        };
        Self {
            account_handle,
            // by default we synchronize from the latest address (supposedly unspent)
            address_index: latest_address_index,
            gap_limit: if latest_address_index == 0 {
                default_gap_limit
            } else {
                1
            },
            skip_persistence: false,
            skip_change_addresses: false,
            steps: vec![
                AccountSynchronizeStep::SyncAddresses(None),
                AccountSynchronizeStep::SyncMessages,
            ],
        }
    }

    /// Number of address indexes that are generated.
    pub fn gap_limit(mut self, limit: usize) -> Self {
        self.gap_limit = limit;
        self
    }

    /// Skip saving new messages and addresses on the account object.
    /// The found data is returned on the `execute` call but won't be persisted on the database.
    pub fn skip_persistence(mut self) -> Self {
        self.skip_persistence = true;
        self
    }

    /// Skip syncing existing change addresses.
    pub fn skip_change_addresses(mut self) -> Self {
        self.skip_change_addresses = true;
        self
    }

    /// Initial address index to start syncing.
    pub fn address_index(mut self, address_index: usize) -> Self {
        self.address_index = address_index;
        self
    }

    /// Sets the steps to run on the sync process.
    /// By default it runs all steps (sync_addresses and sync_messages),
    /// but the library can pick what to run here.
    pub(crate) fn steps(mut self, steps: Vec<AccountSynchronizeStep>) -> Self {
        self.steps = steps;
        self
    }

    pub(crate) async fn get_new_history(&self, return_all_addresses: bool) -> crate::Result<SyncedAccountData> {
        log::debug!("get_new_history");
        let change_addresses_to_sync = self.account_handle.change_addresses_to_sync.lock().await.clone();
        perform_sync(
            self.account_handle.clone(),
            self.address_index,
            self.gap_limit,
            self.skip_change_addresses,
            change_addresses_to_sync,
            &self.steps,
            self.account_handle.account_options,
            return_all_addresses,
        )
        .await
    }

    pub(crate) async fn get_events(
        account_options: AccountOptions,
        addresses_before_sync: &[(String, u64, HashMap<OutputId, AddressOutput>)],
        addresses: &[Address],
        new_messages: &[Message],
        confirmation_changed_messages: &[Message],
    ) -> crate::Result<SyncedAccountEvents> {
        log::debug!("get_events");
        // balance event
        let mut balance_change_events = Vec::new();
        for address_after_sync in addresses.iter() {
            let address_bech32 = address_after_sync.address().to_bech32();
            let (address_before_sync, before_sync_balance, before_sync_outputs) = addresses_before_sync
                .iter()
                .find(|(address, _, _)| &address_bech32 == address)
                .cloned()
                .unwrap_or_else(|| (address_bech32, 0, HashMap::new()));
            if address_after_sync.balance() != before_sync_balance {
                log::debug!(
                    "[SYNC] address {} balance changed from {} to {}",
                    address_before_sync,
                    before_sync_balance,
                    address_after_sync.balance()
                );
                balance_change_events.extend(get_balance_change_events(
                    before_sync_balance,
                    address_after_sync.balance(),
                    address_after_sync.address().clone(),
                    account_options,
                    before_sync_outputs,
                    address_after_sync.outputs(),
                ))
            }
        }

        // new messages event
        let mut new_transaction_events = Vec::new();
        for message in new_messages.iter() {
            log::info!("[SYNC] new message: {:?}", message.id());
            new_transaction_events.push(message.clone());
        }

        // confirmation state change event
        let mut confirmation_change_events = Vec::new();
        for message in confirmation_changed_messages.iter() {
            log::info!("[SYNC] message confirmation state changed: {:?}", message.id());
            confirmation_change_events.push(ConfirmationChangeEventData {
                message: message.clone(),
                confirmed: message.confirmed().unwrap_or(false),
            });
        }

        Ok(SyncedAccountEvents {
            balance_change_events,
            new_transaction_events,
            confirmation_change_events,
        })
    }

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    pub async fn execute(self) -> crate::Result<SyncedAccount> {
        log::debug!("[SYNC] execute");
        self.account_handle.disable_mqtt();
        let syc_start_time = std::time::Instant::now();
        let return_value = match self.get_new_history(false).await {
            Ok(data) => {
                let is_empty = data
                    .addresses
                    .iter()
                    .all(|address| address.balance() == 0 && address.outputs().is_empty());
                log::debug!("[SYNC] is empty: {}", is_empty);
                let mut account = self.account_handle.write().await;
                let messages_before_sync: Vec<(MessageId, Option<bool>)> = account
                    .with_messages(|messages| messages.iter().map(|m| (m.key, m.confirmed)).collect())
                    .await;
                let addresses_before_sync: Vec<(String, u64, HashMap<OutputId, AddressOutput>)> = account
                    .addresses()
                    .iter()
                    .map(|a| (a.address().to_bech32(), a.balance(), a.outputs().clone()))
                    .collect();

                let parsed_messages = data
                    .parse_messages(self.account_handle.accounts.clone(), &account)
                    .await?;
                log::debug!("[SYNC] new messages: {:#?}", parsed_messages);
                let new_addresses = data.addresses;
                log::debug!("[SYNC] new addresses: {:#?}", new_addresses);

                if !self.skip_persistence && (!new_addresses.is_empty() || !parsed_messages.is_empty()) {
                    account.append_addresses(new_addresses.to_vec());
                    account.save_messages(parsed_messages.to_vec()).await?;
                    account.set_last_synced_at(Some(chrono::Local::now()));
                    account.save().await?;
                }

                let mut new_messages = Vec::new();
                let mut confirmation_changed_messages = Vec::new();
                for message in parsed_messages {
                    if !messages_before_sync.iter().any(|(id, _)| id == message.id()) {
                        new_messages.push(message.clone());
                    }
                    if messages_before_sync
                        .iter()
                        .any(|(id, confirmed)| id == message.id() && confirmed != message.confirmed())
                    {
                        confirmation_changed_messages.push(message);
                    }
                }

                let persist_events = self.account_handle.account_options.persist_events;
                let events = Self::get_events(
                    self.account_handle.account_options,
                    &addresses_before_sync,
                    &new_addresses,
                    &new_messages,
                    &confirmation_changed_messages,
                )
                .await?;
                for message in events.new_transaction_events {
                    emit_transaction_event(TransactionEventType::NewTransaction, &account, message, persist_events)
                        .await?;
                }
                for confirmation_change_event in events.confirmation_change_events {
                    emit_confirmation_state_change(
                        &account,
                        confirmation_change_event.message,
                        confirmation_change_event.confirmed,
                        persist_events,
                    )
                    .await?;
                }
                for balance_change_event in events.balance_change_events {
                    emit_balance_change(
                        &account,
                        &balance_change_event.address,
                        balance_change_event.message_id,
                        balance_change_event.balance_change,
                        persist_events,
                    )
                    .await?;
                }

                let mut updated_messages = new_messages;
                updated_messages.extend(confirmation_changed_messages);

                let synced_account = SyncedAccount {
                    id: account.id().to_string(),
                    index: *account.index(),
                    account_handle: self.account_handle.clone(),
                    deposit_address: account.latest_address().clone(),
                    is_empty,
                    addresses: new_addresses,
                    messages: updated_messages,
                };
                log::debug!("[SYNC] syncing took: {:.2?}", syc_start_time.elapsed());
                Ok(synced_account)
            }
            Err(e) => {
                log::debug!("[SYNC] get_new_history error {}", e);
                Err(e)
            }
        };

        self.account_handle.enable_mqtt();

        return_value
    }
}

/// Data returned from account synchronization.
#[derive(Debug, Clone, Getters, Serialize)]
pub struct SyncedAccount {
    /// The account identifier.
    id: String,
    /// The account index.
    index: usize,
    /// The associated account handle.
    #[serde(skip)]
    #[getset(get = "pub")]
    pub(crate) account_handle: AccountHandle,
    /// The account's deposit address.
    #[serde(rename = "depositAddress")]
    #[getset(get = "pub")]
    deposit_address: Address,
    /// Whether the synced account is empty or not.
    #[serde(rename = "isEmpty")]
    #[getset(get = "pub(crate)")]
    is_empty: bool,
    /// The newly found and updated account messages.
    #[getset(get = "pub")]
    pub(crate) messages: Vec<Message>,
    /// The newly generated and updated account addresses.
    #[getset(get = "pub")]
    pub(crate) addresses: Vec<Address>,
}

#[derive(Debug, Clone, Getters)]
pub(crate) struct SyncedAccountEvents {
    pub(crate) balance_change_events: Vec<BalanceChangeEventData>,
    pub(crate) new_transaction_events: Vec<Message>,
    pub(crate) confirmation_change_events: Vec<ConfirmationChangeEventData>,
}

impl SyncedAccount {
    /// Emulates a synced account from an account handle.
    /// Should only be used if sync is guaranteed (e.g. when using MQTT)
    pub(crate) async fn from(account_handle: AccountHandle) -> Self {
        let id = account_handle.id().await;
        let index = account_handle.index().await;
        let deposit_address = account_handle.latest_address().await;
        Self {
            id,
            index,
            deposit_address,
            account_handle,
            is_empty: false,
            messages: Default::default(),
            addresses: Default::default(),
        }
    }

    /// Selects input addresses for a value transaction.
    /// The method ensures that the recipient address doesn’t match the remainder address.
    ///
    /// # Arguments
    ///
    /// * `threshold` Amount user wants to spend.
    /// * `address` Recipient address.
    ///
    /// # Return value
    ///
    /// Returns a (addresses, address) tuple representing the selected input addresses and the remainder address if
    /// needed.
    fn select_inputs<'a>(
        &self,
        locked_outputs: &'a mut MutexGuard<'_, Vec<AddressOutput>>,
        transfer_obj: &Transfer,
        available_outputs: Vec<input_selection::AddressInputs>,
        signer_type: SignerType,
    ) -> crate::Result<(Vec<input_selection::AddressInputs>, Option<input_selection::Remainder>)> {
        let output_amount = transfer_obj.outputs.len();
        let max_inputs = match signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => {
                // -1 because we need at least one input and the limit is for inputs and outputs together
                if output_amount >= LEDGER_MAX_IN_OUTPUTS - 1 {
                    return Err(crate::Error::TooManyOutputs(output_amount, LEDGER_MAX_IN_OUTPUTS - 1));
                }
                LEDGER_MAX_IN_OUTPUTS - output_amount
            }
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => {
                // -1 because we need at least one input and the limit is for inputs and outputs together
                if output_amount >= LEDGER_MAX_IN_OUTPUTS - 1 {
                    return Err(crate::Error::TooManyOutputs(output_amount, LEDGER_MAX_IN_OUTPUTS - 1));
                }
                LEDGER_MAX_IN_OUTPUTS - output_amount
            }
            _ => {
                if output_amount >= INPUT_OUTPUT_COUNT_MAX {
                    return Err(crate::Error::TooManyOutputs(output_amount, INPUT_OUTPUT_COUNT_MAX));
                }
                INPUT_OUTPUT_COUNT_MAX
            }
        };

        let mut available_inputs: Vec<input_selection::Input> = Vec::new();
        for address_input in available_outputs {
            let filtered: Vec<AddressOutput> = address_input.clone()
                .outputs
                .clone()
                .into_iter()
                .filter(|output| {
                    (!transfer_obj.outputs.iter().any(|transfer_output| transfer_output.address == output.address)
                        && *output.amount() > 0
                        && !locked_outputs.iter().any(|locked_output| locked_output.transaction_id == output.transaction_id && locked_output.index == output.index)
                        // we allow an input equal to a deposit address only if it has balance <= transfer amount, so there
                        // can't be a remainder value with this address as input alone
                    || transfer_obj.outputs.iter().any(|o| &o.address == output.address())
                        && *output.amount() <= transfer_obj.amount())
                        && *output.amount() > 0
                        && !locked_outputs.iter().any(|locked_output| {
                            locked_output.transaction_id == output.transaction_id && locked_output.index == output.index
                        })
                }).collect();
            for output in filtered {
                available_inputs.push(input_selection::Input {
                    internal: address_input.internal,
                    output: output.clone(),
                });
            }
        }

        let selected_outputs = input_selection::select_input(transfer_obj.amount(), available_inputs, max_inputs)?;
        locked_outputs.extend(selected_outputs.iter().map(|input| input.output.clone()));

        let inputs_amount = selected_outputs.iter().fold(0, |acc, a| acc + a.output.amount);
        let has_remainder = inputs_amount > transfer_obj.amount();

        let remainder = if has_remainder {
            let input_for_remainder = selected_outputs
                .iter()
                // We filter the output addresses, but since we checked that this address balance <=
                // transfer_obj.amount.get() we need to have another input address
                .filter(|input| !transfer_obj.outputs.iter().any(|o| o.address == input.output.address))
                .collect::<Vec<&input_selection::Input>>()
                .last()
                .cloned()
                .cloned();
            if let Some(remainder) = input_for_remainder {
                Some(input_selection::Remainder {
                    address: remainder.output.address,
                    internal: remainder.internal,
                    amount: inputs_amount,
                })
            } else {
                return Err(crate::Error::FailedToGetRemainder);
            }
        } else {
            None
        };
        let mut selected_address_outputs: HashMap<AddressWrapper, input_selection::AddressInputs> = HashMap::new();
        for input in selected_outputs {
            match selected_address_outputs.get_mut(&input.output.address) {
                Some(entry) => entry.outputs.push(input.output),
                None => {
                    selected_address_outputs.insert(
                        input.output.address.clone(),
                        input_selection::AddressInputs {
                            address: input.output.address.clone(),
                            internal: input.internal,
                            outputs: vec![input.output.clone()],
                        },
                    );
                }
            }
        }

        Ok((
            selected_address_outputs
                .into_iter()
                .map(|(_id, address_inputs)| address_inputs)
                .collect(),
            remainder,
        ))
    }

    async fn get_output_consolidation_transfers(
        &self,
        include_dust_allowance_outputs: bool,
    ) -> crate::Result<Vec<Transfer>> {
        let mut transfers: Vec<Transfer> = Vec::new();
        // collect the transactions we need to make
        {
            let account = self.account_handle.read().await;
            let sent_messages = account.list_messages(0, 0, Some(MessageType::Sent)).await?;
            for address in account.addresses() {
                if address.outputs().len() >= self.account_handle.account_options.output_consolidation_threshold {
                    let mut address_outputs = address.available_outputs(&sent_messages);
                    if !include_dust_allowance_outputs {
                        address_outputs = address_outputs
                            .into_iter()
                            .filter(|addr| addr.kind != OutputKind::SignatureLockedDustAllowance)
                            .collect();
                    }

                    // the address outputs exceed the threshold, so we push a transfer to our vector
                    if address_outputs.len() >= self.account_handle.account_options.output_consolidation_threshold {
                        // take hardware limits of ledger nano into account
                        let max_inputs = match account.signer_type {
                            #[cfg(feature = "ledger-nano")]
                            SignerType::LedgerNano => LEDGER_MAX_IN_OUTPUTS - 1,
                            #[cfg(feature = "ledger-nano-simulator")]
                            SignerType::LedgerNanoSimulator => LEDGER_MAX_IN_OUTPUTS - 1,
                            _ => INPUT_OUTPUT_COUNT_MAX - 1,
                        };
                        for outputs in address_outputs.chunks(max_inputs) {
                            // Only create dust_allowance_output if an input is also a dust_allowance_outputs
                            let output_kind = if include_dust_allowance_outputs
                                && outputs
                                    .iter()
                                    .any(|addr| addr.kind == OutputKind::SignatureLockedDustAllowance)
                            {
                                Some(OutputKind::SignatureLockedDustAllowance)
                            } else {
                                None
                            };
                            transfers.push(
                                Transfer::builder(
                                    address.address().clone(),
                                    NonZeroU64::new(outputs.iter().fold(0, |v, o| v + o.amount)).unwrap(),
                                    output_kind,
                                )
                                .with_input(
                                    address.address().clone(),
                                    outputs.iter().map(|o| (*o).clone()).collect(),
                                )
                                .with_events(false)
                                .finish(),
                            );
                        }
                    }
                }
            }
        }
        Ok(transfers)
    }

    /// Consolidate account outputs.
    pub(crate) async fn consolidate_outputs(
        &self,
        include_dust_allowance_outputs: bool,
    ) -> crate::Result<Vec<Message>> {
        log::debug!("consolidate_outputs");
        let mut tasks = Vec::new();
        // run the transfers in parallel
        for transfer in self
            .get_output_consolidation_transfers(include_dust_allowance_outputs)
            .await?
        {
            let task = self.transfer(transfer);
            tasks.push(task);
        }

        let mut messages = Vec::new();
        for message in futures::future::try_join_all(tasks).await? {
            messages.push(message);
        }

        Ok(messages)
    }

    #[cfg(feature = "participation")]
    /// Gets all outputs and creates transactions to send them to an own address again
    pub(crate) async fn send_participation_transfers(
        &self,
        mut participations: Vec<crate::participation::types::Participation>,
        custom_inputs: Option<Vec<AddressOutput>>,
    ) -> crate::Result<Vec<Message>> {
        let mut transfers: Vec<Transfer> = Vec::new();
        // collect the transactions we need to make

        let account = self.account_handle.read().await;
        if let Ok(read_participations) = crate::storage::get(&account.storage_path)
            .await?
            .lock()
            .await
            .get_participations(*account.index())
            .await
        {
            // add existing participations
            for participation in read_participations {
                if !participations.iter().any(|p| p.event_id == participation.event_id) {
                    participations.push(participation);
                }
            }
        }
        // -1 because we will generate one output
        let max_inputs = match account.signer_type {
            #[cfg(feature = "ledger-nano")]
            SignerType::LedgerNano => LEDGER_MAX_IN_OUTPUTS - 1,
            #[cfg(feature = "ledger-nano-simulator")]
            SignerType::LedgerNanoSimulator => LEDGER_MAX_IN_OUTPUTS - 1,
            _ => INPUT_OUTPUT_COUNT_MAX - 1,
        };
        let available_outputs = match custom_inputs {
            Some(inputs) => inputs,
            None => {
                let sent_messages = account.list_messages(0, 0, Some(MessageType::Sent)).await?;
                let mut available_outputs: Vec<AddressOutput> = Vec::new();
                for address in account.addresses() {
                    let address_outputs = address.available_outputs(&sent_messages);
                    available_outputs.extend(address_outputs.into_iter().cloned());
                }
                available_outputs
            }
        };

        log::debug!("Participation: {:?}", participations);
        let indexation_payload = if participations.is_empty() {
            crate::message::IndexationPayload::new("firefly".as_bytes(), &[])?
        } else {
            crate::message::IndexationPayload::new(
                crate::participation::types::PARTICIPATE.as_bytes(),
                &crate::participation::types::Participations {
                    participations: participations.clone(),
                }
                .to_bytes()?,
            )?
        };
        // the address outputs exceed the threshold, so we push a transfer to our vector
        if !available_outputs.is_empty() {
            for outputs in available_outputs.chunks(max_inputs) {
                transfers.push(
                    Transfer::builder(
                        // save to unwrap since we checked that it's not empty
                        outputs.first().unwrap().address.clone(),
                        NonZeroU64::new(outputs.iter().fold(0, |v, o| v + o.amount)).unwrap(),
                        None,
                    )
                    .with_inputs(outputs.iter().map(|o| (*o).clone()).collect())
                    .with_events(true)
                    .with_indexation(indexation_payload.clone())
                    .finish(),
                );
            }
        } else {
            return Err(crate::Error::InsufficientFunds(0, 0));
        }
        let account_id = account.id().to_string();
        drop(account);

        log::debug!("send participation transfers");
        let mut tasks = Vec::new();
        // run the transfers in parallel
        for transfer in transfers {
            transfer
                .emit_event_if_needed(account_id.clone(), TransferProgressType::SelectingInputs)
                .await;
            let task = self.transfer(transfer);
            tasks.push(task);
        }

        let mut messages = Vec::new();
        for message in futures::future::try_join_all(tasks).await? {
            messages.push(message);
        }

        let account = self.account_handle.read().await;
        crate::storage::get(&account.storage_path)
            .await?
            .lock()
            .await
            .save_participations(*account.index(), participations)
            .await?;

        Ok(messages)
    }

    /// Send messages.
    pub(crate) async fn transfer(&self, mut transfer_obj: Transfer) -> crate::Result<Message> {
        log::debug!("[TRANSFER] transfer");
        let account_ = self.account_handle.read().await;

        // validate ledger seed for ledger accounts
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        {
            let ledger = match account_.signer_type() {
                #[cfg(feature = "ledger-nano")]
                SignerType::LedgerNano => true,
                #[cfg(feature = "ledger-nano-simulator")]
                SignerType::LedgerNanoSimulator => true,
                _ => false,
            };
            // validate that the first address matches the first address of the account, validation happens inside of
            // get_address_with_index
            if ledger {
                log::debug!("[TRANSFER] validate ledger seed with first address");
                let _ = crate::address::get_address_with_index(
                    &account_,
                    0,
                    account_.bech32_hrp(),
                    GenerateAddressMetadata {
                        syncing: true,
                        network: account_.network(),
                    },
                )
                .await?;
            }
        }

        // if any of the deposit addresses belongs to the account, we'll reuse the input address
        // for remainder value output. This is the only way to know the transaction value for
        // transactions between account addresses.
        if account_
            .addresses()
            .iter()
            .any(|a| transfer_obj.outputs.iter().any(|o| &o.address == a.address()))
        {
            transfer_obj.remainder_value_strategy = RemainderValueStrategy::ReuseAddress;
        }

        // lock the transfer process until we select the input (outputs)
        // we do this to prevent multiple threads trying to transfer at the same time
        // so it doesn't consume the same outputs multiple times, which leads to a conflict state
        let account_outputs_locker = self.account_handle.locked_outputs.clone();
        let mut locked_outputs = account_outputs_locker.lock().await;

        // prepare the transfer getting some needed objects and values
        let value = transfer_obj.amount();

        let sent_messages = account_.list_messages(0, 0, Some(MessageType::Sent)).await?;

        let balance = account_.balance_internal(&sent_messages).await;

        if value > balance.total {
            return Err(crate::Error::InsufficientFunds(balance.total, value));
        }

        if let RemainderValueStrategy::AccountAddress(ref remainder_deposit_address) =
            transfer_obj.remainder_value_strategy
        {
            if !account_
                .addresses()
                .iter()
                .any(|addr| addr.address() == remainder_deposit_address)
            {
                return Err(crate::Error::InvalidRemainderValueAddress);
            }
        }

        let (input_addresses, remainder_address): (
            Vec<input_selection::AddressInputs>,
            Option<input_selection::Remainder>,
        ) = match transfer_obj.input.take() {
            Some(addresses_inputs) => {
                let mut address_inputs = Vec::new();
                for address_input in addresses_inputs {
                    if let Some(address) = account_.addresses().iter().find(|a| a.address() == &address_input.0) {
                        locked_outputs.extend(address_input.1.iter().cloned());
                        address_inputs.push(input_selection::AddressInputs {
                            internal: *address.internal(),
                            address: address.address().clone(),
                            outputs: address_input.1,
                        });
                    } else {
                        return Err(crate::Error::InputAddressNotFound);
                    }
                }
                (address_inputs, None)
            }
            None => {
                transfer_obj
                    .emit_event_if_needed(account_.id().to_string(), TransferProgressType::SelectingInputs)
                    .await;
                // Get all available outputs
                let available_outputs = account_
                    .addresses()
                    .iter()
                    .map(|address| input_selection::AddressInputs {
                        address: address.address().clone(),
                        internal: *address.internal(),
                        outputs: address
                            .available_outputs(&sent_messages)
                            .iter()
                            .map(|o| (*o).clone())
                            .collect::<Vec<AddressOutput>>(),
                    })
                    .collect();

                let signer_type = account_.signer_type().clone();

                // select the input addresses and check if a remainder address is needed
                let (selected_inputs, remainder_address) =
                    self.select_inputs(&mut locked_outputs, &transfer_obj, available_outputs, signer_type)?;
                (selected_inputs, remainder_address)
            }
        };

        // unlock the transfer process since we already selected the input addresses and locked them
        drop(locked_outputs);
        drop(account_);

        log::debug!(
            "[TRANSFER] inputs: {:#?} - remainder address: {:?}",
            input_addresses,
            remainder_address
        );

        let res = perform_transfer(
            transfer_obj,
            &input_addresses,
            self.account_handle.clone(),
            remainder_address,
        )
        .await;

        let mut locked_outputs = account_outputs_locker.lock().await;
        for input_address in &input_addresses {
            let index = locked_outputs
                .iter()
                .position(|a| {
                    input_address
                        .outputs
                        .iter()
                        .any(|output| output.transaction_id == a.transaction_id && output.index == a.index)
                })
                .unwrap();
            locked_outputs.remove(index);
        }

        res
    }

    /// Retry message.
    pub(crate) async fn retry(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Retry).await
    }

    /// Promote message.
    pub(crate) async fn promote(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Promote).await
    }

    /// Reattach message.
    pub(crate) async fn reattach(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Reattach).await
    }
}

async fn perform_transfer(
    transfer_obj: Transfer,
    input_addresses: &[input_selection::AddressInputs],
    account_handle: AccountHandle,
    remainder_address: Option<input_selection::Remainder>,
) -> crate::Result<Message> {
    log::debug!("[TRANSFER] perform_transfer");
    let mut utxos = vec![];
    let mut transaction_inputs = vec![];
    // store (amount, address, new_created) to check later if dust is allowed
    let mut dust_and_allowance_recorders = Vec::new();
    let transfer_amount = transfer_obj.amount();

    let mut outputs_for_event: Vec<TransactionIO> = Vec::new();
    for output in transfer_obj.outputs.iter() {
        if transfer_amount < DUST_ALLOWANCE_VALUE {
            dust_and_allowance_recorders.push((output.amount.get(), output.address.to_bech32(), true));
        }
        outputs_for_event.push(TransactionIO {
            address: output.address.to_bech32(),
            amount: u64::from(output.amount),
            remainder: Some(false),
        });
    }
    // do we need to add dust_allowance to dust_and_allowance_recorders here?

    let account_ = account_handle.read().await;

    for address_input in input_addresses.iter() {
        let account_address = account_
            .addresses()
            .iter()
            .find(|a| a.address() == &address_input.address)
            .unwrap();

        let mut outputs = vec![];

        for address_output in address_input.outputs.iter() {
            outputs.push((
                address_output.clone(),
                *account_address.key_index(),
                *account_address.internal(),
            ));
        }
        utxos.extend(outputs.into_iter());
    }

    let mut outputs_for_essence: Vec<Output> = Vec::new();
    for output in transfer_obj.outputs.iter() {
        match output.output_kind {
            crate::address::OutputKind::SignatureLockedSingle => {
                outputs_for_essence
                    .push(SignatureLockedSingleOutput::new(*output.address.as_ref(), output.amount.get())?.into());
            }
            crate::address::OutputKind::SignatureLockedDustAllowance => {
                outputs_for_essence.push(
                    SignatureLockedDustAllowanceOutput::new(*output.address.as_ref(), output.amount.get())?.into(),
                );
            }
            _ => return Err(crate::error::Error::InvalidOutputKind("Treasury".to_string())),
        }
    }
    let mut inputs_for_essence: Vec<Input> = Vec::new();
    let mut inputs_for_event: Vec<TransactionIO> = Vec::new();
    let mut current_output_sum = 0;
    let mut remainder_value = 0;

    for (utxo, address_index, address_internal) in utxos {
        let (amount, address) = match utxo.kind {
            OutputKind::SignatureLockedSingle => {
                if utxo.amount < DUST_ALLOWANCE_VALUE {
                    dust_and_allowance_recorders.push((utxo.amount, utxo.address.to_bech32(), false));
                }
                (utxo.amount, utxo.address.to_bech32())
            }
            OutputKind::SignatureLockedDustAllowance => {
                dust_and_allowance_recorders.push((utxo.amount, utxo.address.to_bech32(), false));
                (utxo.amount, utxo.address.to_bech32())
            }
            OutputKind::Treasury => return Err(crate::Error::InvalidOutputKind("Treasury".to_string())),
        };
        inputs_for_event.push(TransactionIO {
            address,
            amount,
            remainder: None,
        });

        let input: Input = UtxoInput::new(*utxo.transaction_id(), *utxo.index())?.into();
        inputs_for_essence.push(input.clone());
        transaction_inputs.push(crate::signing::TransactionInput {
            input,
            address_index,
            address_internal,
        });
        if current_output_sum == transfer_amount {
            log::debug!(
                    "[TRANSFER] current output sum matches the transfer value, adding {} to the remainder value (currently at {})",
                    utxo.amount(),
                    remainder_value
                );
            // already filled the transfer value; just collect the output value as remainder
            remainder_value += *utxo.amount();
        } else if current_output_sum + *utxo.amount() > transfer_amount {
            log::debug!(
                "[TRANSFER] current output sum ({}) would exceed the transfer value if added to the output amount ({})",
                current_output_sum,
                utxo.amount()
            );
            // if the used UTXO amount is greater than the transfer value,
            // this is the last iteration and we'll have remainder value
            let missing_value = transfer_amount - current_output_sum;
            remainder_value += *utxo.amount() - missing_value;
            current_output_sum += missing_value;
            log::debug!(
                "[TRANSFER] added output with the missing value {}, and the remainder is {}",
                missing_value,
                remainder_value
            );

            let remaining_balance_on_source = current_output_sum - transfer_amount;
            if remaining_balance_on_source < DUST_ALLOWANCE_VALUE && remaining_balance_on_source != 0 {
                dust_and_allowance_recorders.push((remaining_balance_on_source, utxo.address().to_bech32(), true));
            }
        } else {
            log::debug!(
                "[TRANSFER] adding output amount {}, current sum {}",
                utxo.amount(),
                current_output_sum
            );
            current_output_sum += *utxo.amount();

            if current_output_sum > transfer_amount {
                let remaining_balance_on_source = current_output_sum - transfer_amount;
                if remaining_balance_on_source < DUST_ALLOWANCE_VALUE && remaining_balance_on_source != 0 {
                    dust_and_allowance_recorders.push((remaining_balance_on_source, utxo.address().to_bech32(), true));
                }
            }
        }
    }

    drop(account_);
    let mut account_ = account_handle.write().await;

    let mut addresses_to_watch = vec![];

    // if there's remainder value, we check the strategy defined in the transfer
    let mut remainder_value_deposit_address = None;
    let remainder_deposit_address = if remainder_value > 0 {
        let remainder_address = remainder_address.as_ref().expect("remainder address not defined");
        let remainder_address = account_
            .addresses()
            .iter()
            .find(|a| a.address() == &remainder_address.address)
            .unwrap();

        log::debug!("[TRANSFER] remainder value is {}", remainder_value);

        let remainder_deposit_address = match transfer_obj.remainder_value_strategy.clone() {
            // use one of the account's addresses to send the remainder value
            RemainderValueStrategy::AccountAddress(target_address) => {
                log::debug!(
                    "[TARGET] using user defined account address as remainder target: {}",
                    target_address.to_bech32()
                );
                target_address
            }
            // generate a new change address to send the remainder value
            RemainderValueStrategy::ChangeAddress => {
                let change_address = if let Some(address) = account_.latest_change_address() {
                    if address.outputs().is_empty() {
                        log::debug!(
                            "[TRANSFER] using latest latest_change_address as remainder target: {}",
                            address.address().to_bech32()
                        );
                        transfer_obj
                            .emit_event_if_needed(
                                account_.id().to_string(),
                                TransferProgressType::GeneratingRemainderDepositAddress(AddressData {
                                    address: address.address().to_bech32(),
                                }),
                            )
                            .await;
                        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
                        {
                            let ledger = match account_.signer_type() {
                                #[cfg(feature = "ledger-nano")]
                                SignerType::LedgerNano => true,
                                #[cfg(feature = "ledger-nano-simulator")]
                                SignerType::LedgerNanoSimulator => true,
                                _ => false,
                            };
                            if ledger {
                                log::debug!("[TRANSFER] regnerate address so it's displayed on the ledger");
                                let regenerated_address = crate::address::get_new_change_address(
                                    &account_,
                                    *address.key_index(),
                                    account_.bech32_hrp(),
                                    GenerateAddressMetadata {
                                        syncing: false,
                                        network: account_.network(),
                                    },
                                )
                                .await?;
                                if address.address().inner != regenerated_address.address().inner {
                                    return Err(crate::Error::LedgerMnemonicMismatch);
                                }
                            }
                        }
                        address.clone()
                    } else {
                        let address = crate::address::get_new_change_address(
                            &account_,
                            // Index +1 because we want a new address
                            address.key_index() + 1,
                            account_.bech32_hrp(),
                            GenerateAddressMetadata {
                                syncing: true,
                                network: account_.network(),
                            },
                        )
                        .await?;
                        log::debug!(
                            "[TRANSFER] generated new change address as remainder target: {}",
                            address.address().to_bech32()
                        );
                        transfer_obj
                            .emit_event_if_needed(
                                account_.id().to_string(),
                                TransferProgressType::GeneratingRemainderDepositAddress(AddressData {
                                    address: address.address().to_bech32(),
                                }),
                            )
                            .await;
                        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
                        {
                            let ledger = match account_.signer_type() {
                                #[cfg(feature = "ledger-nano")]
                                SignerType::LedgerNano => true,
                                #[cfg(feature = "ledger-nano-simulator")]
                                SignerType::LedgerNanoSimulator => true,
                                _ => false,
                            };
                            if ledger {
                                log::debug!("[TRANSFER] regnerate address so it's displayed on the ledger");
                                let regenerated_address = crate::address::get_new_change_address(
                                    &account_,
                                    *address.key_index(),
                                    account_.bech32_hrp(),
                                    GenerateAddressMetadata {
                                        syncing: false,
                                        network: account_.network(),
                                    },
                                )
                                .await?;
                                if address.address().inner != regenerated_address.address().inner {
                                    return Err(crate::Error::LedgerMnemonicMismatch);
                                }
                            }
                        }
                        address
                    }
                } else {
                    // Generate an address with syncing: true so it doesn't get displayed, then generate it with
                    // syncing:false so the user can verify it on the ledger
                    let change_address_for_event = crate::address::get_new_change_address(
                        &account_,
                        // Index 0 because it's the first address
                        0,
                        account_.bech32_hrp(),
                        GenerateAddressMetadata {
                            syncing: true,
                            network: account_.network(),
                        },
                    )
                    .await?;
                    transfer_obj
                        .emit_event_if_needed(
                            account_.id().to_string(),
                            TransferProgressType::GeneratingRemainderDepositAddress(AddressData {
                                address: change_address_for_event.address().to_bech32(),
                            }),
                        )
                        .await;
                    let change_address = crate::address::get_new_change_address(
                        &account_,
                        // Index 0 because it's the first address
                        0,
                        account_.bech32_hrp(),
                        GenerateAddressMetadata {
                            syncing: false,
                            network: account_.network(),
                        },
                    )
                    .await?;
                    log::debug!(
                        "[TRANSFER] generated new change address as remainder target: {}",
                        change_address.address().to_bech32()
                    );
                    change_address
                };
                account_.append_addresses(vec![change_address.clone()]);
                account_.save().await?;
                addresses_to_watch.push(change_address.address().clone());

                account_handle
                    .change_addresses_to_sync
                    .lock()
                    .await
                    .insert(change_address.address().clone());
                change_address.address().clone()
            }
            // keep the remainder value on the address
            RemainderValueStrategy::ReuseAddress => {
                let address = remainder_address.address().clone();
                log::debug!("[TRANSFER] reusing address as remainder target {}", address.to_bech32());
                address
            }
        };
        remainder_value_deposit_address.replace(remainder_deposit_address.clone());
        outputs_for_essence
            .push(SignatureLockedSingleOutput::new(*remainder_deposit_address.as_ref(), remainder_value)?.into());
        Some(remainder_deposit_address)
    } else {
        None
    };

    if let Some(remainder_deposit_address) = &remainder_deposit_address {
        if remainder_value < DUST_ALLOWANCE_VALUE {
            dust_and_allowance_recorders.push((remainder_value, remainder_deposit_address.to_bech32(), true));
        }
        outputs_for_event.push(TransactionIO {
            address: remainder_deposit_address.to_bech32(),
            amount: remainder_value,
            remainder: Some(true),
        });
    }

    let client = crate::client::get_client(account_.client_options()).await?;
    let client = client.read().await;

    // Check if we would let dust on an address behind or send new dust, which would make the tx unconfirmable
    let mut single_addresses = HashSet::new();
    for dust_or_allowance in &dust_and_allowance_recorders {
        single_addresses.insert(dust_or_allowance.1.to_string());
    }
    for address in single_addresses {
        let created_or_consumed_outputs: Vec<(u64, bool)> = dust_and_allowance_recorders
            .iter()
            .filter(|d| d.1 == address)
            .map(|(amount, _, flag)| (*amount, *flag))
            .collect();
        is_dust_allowed(&account_, &client, address, created_or_consumed_outputs).await?;
    }

    // Build transaction essence
    let mut essence_builder = RegularEssence::builder();

    // Order inputs and add them to the essence
    inputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_inputs(inputs_for_essence);

    // Order outputs and add them to the essence
    outputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_outputs(outputs_for_essence);

    let mut indexation_data = None;
    if let Some(indexation) = &transfer_obj.indexation {
        if !indexation.data().is_empty() {
            indexation_data = Some(hex::encode(&*indexation.data()));
        }
        essence_builder = essence_builder.with_payload(Payload::Indexation(Box::new(indexation.clone())));
    }

    let essence = essence_builder.finish()?;
    let essence = Essence::Regular(essence);

    transfer_obj
        .emit_event_if_needed(
            account_.id().to_string(),
            TransferProgressType::PreparedTransaction(PreparedTransactionData {
                inputs: inputs_for_event,
                outputs: outputs_for_event,
                data: indexation_data.clone(),
            }),
        )
        .await;
    transfer_obj
        .emit_event_if_needed(account_.id().to_string(), TransferProgressType::SigningTransaction)
        .await;
    let unlock_blocks = crate::signing::get_signer(account_.signer_type())
        .await
        .lock()
        .await
        .sign_message(
            &account_,
            &essence,
            &mut transaction_inputs,
            SignMessageMetadata {
                remainder_address: remainder_address.map(|remainder| {
                    account_
                        .addresses()
                        .iter()
                        .find(|a| a.address() == &remainder.address)
                        .unwrap()
                }),
                remainder_value,
                remainder_deposit_address: remainder_deposit_address
                    .map(|address| account_.addresses().iter().find(|a| a.address() == &address).unwrap()),
                network: account_.network(),
            },
        )
        .await?;

    let transaction = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(UnlockBlocks::new(unlock_blocks)?)
        .finish()?;

    transfer_obj
        .emit_event_if_needed(account_.id().to_string(), TransferProgressType::PerformingPoW)
        .await;
    let message = finish_pow(&client, Some(Payload::Transaction(Box::new(transaction)))).await?;

    log::debug!("[TRANSFER] submitting message {:#?}", message);

    transfer_obj
        .emit_event_if_needed(account_.id().to_string(), TransferProgressType::Broadcasting)
        .await;

    let message_id = client.post_message(&message).await?;

    // if this is a transfer to the account's latest address or we used the latest as deposit of the remainder
    // value, we generate a new one to keep the latest address unused
    let latest_address = account_.latest_address().address();
    let latest_address_in_transfer_output = transfer_obj.outputs.iter().any(|o| &o.address == latest_address);
    if latest_address_in_transfer_output
        || (remainder_value_deposit_address.is_some() && &remainder_value_deposit_address.unwrap() == latest_address)
    {
        log::debug!(
            "[TRANSFER] generating new address since {}",
            if latest_address_in_transfer_output {
                "latest address is part of the transfer output"
            } else {
                "latest address equals the remainder value deposit address"
            }
        );
        // We set it to syncing: true so it will not be shown on the ledger
        let addr = crate::address::get_new_address(
            &account_,
            GenerateAddressMetadata {
                syncing: true,
                network: account_.network(),
            },
        )
        .await?;
        addresses_to_watch.push(addr.address().clone());
        account_.append_addresses(vec![addr]);
    }

    // drop the  client ref so it doesn't lock the Message parsing
    drop(client);

    let message = Message::from_iota_message(
        message_id,
        message,
        account_handle.accounts.clone(),
        account_.id(),
        account_.addresses(),
        account_.client_options(),
    )
    .finish()
    .await?;
    account_.save_messages(vec![message.clone()]).await?;
    for input_address in input_addresses {
        if input_address.internal {
            account_handle
                .change_addresses_to_sync
                .lock()
                .await
                .insert(input_address.address.clone());
        }
    }

    // if we generated an address, we need to save the account
    if !addresses_to_watch.is_empty() {
        account_.save().await?;
    }

    // drop the  account_ ref so it doesn't lock the monitor system
    drop(account_);
    crate::monitor::monitor_address_balance(account_handle.clone(), addresses_to_watch).await;

    #[cfg(feature = "participation")]
    {
        // reset all participations if a transfer without participation is sent(indexation_data.is_none()) and the
        // available balance is empty, because then we will have no outputs participating in any event anymore
        if indexation_data.is_none() && account_handle.balance().await?.available == 0 {
            log::debug!("Resetting participations");
            let account = account_handle.read().await;
            let account_index = account_handle.index().await;
            crate::storage::get(&account.storage_path)
                .await?
                .lock()
                .await
                .save_participations(account_index, vec![])
                .await?;
        }
    }
    Ok(message)
}

// Calculate the outputs on this address after the transaction gets confirmed so we know if we can send dust or
// dust allowance outputs (as input). the bool in the outputs defines if we consume this output (false) or create a new
// one (true)
async fn is_dust_allowed(
    account: &Account,
    client: &iota_client::Client,
    address: String,
    outputs: Vec<(u64, bool)>,
) -> crate::Result<()> {
    // balance of all dust allowance outputs
    let mut dust_allowance_balance: i64 = 0;
    // Amount of dust outputs
    let mut dust_outputs_amount: i64 = 0;

    // Add outputs from this transaction
    for (dust, add_outputs) in outputs {
        let sign = if add_outputs { 1 } else { -1 };
        if dust >= DUST_ALLOWANCE_VALUE {
            dust_allowance_balance += sign * dust as i64;
        } else {
            dust_outputs_amount += sign;
        }
    }

    let address_data = client.get_address().balance(&address).await?;
    // If we create a dust output and a dust allowance output we don't need to check more outputs if the balance/100_000
    // is < 100 because then we are sure that we didn't reach the max dust outputs
    if address_data.dust_allowed
        && dust_outputs_amount == 1
        && dust_allowance_balance >= 0
        && address_data.balance as i64 / DUST_DIVISOR < MAX_ALLOWED_DUST_OUTPUTS
    {
        return Ok(());
    } else if !address_data.dust_allowed && dust_outputs_amount == 1 && dust_allowance_balance <= 0 {
        return Err(crate::Error::DustError(format!(
            "No dust output allowed on address {}",
            address
        )));
    }

    // Get outputs from address and apply values
    let address_outputs = if let Some(address) = account.addresses().iter().find(|a| a.address().to_bech32() == address)
    {
        let outputs = address
            .outputs()
            .values()
            .filter(|output| !output.is_spent)
            .map(|output| (output.amount, output.kind.clone()))
            .collect();
        outputs
    } else {
        let outputs = client.find_outputs(&[], &[address.to_string()]).await?;
        let mut address_outputs = Vec::new();
        for output in outputs {
            let output = AddressOutput::from_output_response(output, "".to_string())?;
            address_outputs.push((output.amount, output.kind));
        }
        address_outputs
    };
    for (amount, kind) in address_outputs {
        match kind {
            OutputKind::SignatureLockedDustAllowance => {
                dust_allowance_balance += amount as i64;
            }
            OutputKind::SignatureLockedSingle => {
                if amount < DUST_ALLOWANCE_VALUE {
                    dust_outputs_amount += 1;
                }
            }
            OutputKind::Treasury => {}
        }
    }

    // Here dust_allowance_balance and dust_outputs_amount should be as if this transaction gets confirmed
    // Max allowed dust outputs is 100
    let allowed_dust_amount = std::cmp::min(dust_allowance_balance / DUST_DIVISOR, MAX_ALLOWED_DUST_OUTPUTS);
    if dust_outputs_amount > allowed_dust_amount {
        return Err(crate::Error::DustError(format!(
            "No dust output allowed on address {}",
            address
        )));
    }
    Ok(())
}

pub(crate) enum RepostAction {
    Retry,
    Reattach,
    Promote,
}

pub(crate) async fn repost_message(
    account_handle: AccountHandle,
    message_id: &MessageId,
    action: RepostAction,
) -> crate::Result<Message> {
    let mut account = account_handle.write().await;

    let message = match account.get_message(message_id).await {
        Some(message_to_repost) => {
            let mut message_to_repost = message_to_repost.clone();
            // check all reattachments of the message we want to promote/rettry/reattach
            while let Some(reattachment_message_id) = message_to_repost.reattachment_message_id {
                match account.get_message(&reattachment_message_id).await {
                    Some(m) => {
                        message_to_repost = m.clone();
                        if message_to_repost.confirmed().unwrap_or(false) {
                            return Err(crate::Error::ClientError(Box::new(
                                iota_client::Error::NoNeedPromoteOrReattach(message_id.to_string()),
                            )));
                        }
                    }
                    None => break,
                }
            }

            let client = crate::client::get_client(account.client_options()).await?;
            let client = client.read().await;

            let (id, message) = match action {
                RepostAction::Promote => client.promote(message_id).await?,
                RepostAction::Reattach => client.reattach(message_id).await?,
                RepostAction::Retry => client.retry(message_id).await?,
            };
            let message = Message::from_iota_message(
                id,
                message,
                account_handle.accounts.clone(),
                account.id(),
                account.addresses(),
                account.client_options(),
            )
            .finish()
            .await?;

            account.save_messages(vec![message.clone()]).await?;

            Ok(message)
        }
        None => Err(crate::Error::MessageNotFound),
    }?;

    Ok(message)
}

#[cfg(test)]
mod tests {
    use crate::{
        address::{AddressOutput, OutputKind},
        client::ClientOptionsBuilder,
    };
    use iota_client::bee_message::prelude::{MessageId, TransactionId};
    use quickcheck_macros::quickcheck;
    use std::collections::HashMap;

    #[tokio::test]
    async fn account_sync() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
                .unwrap()
                .build()
                .unwrap();
            let _account = manager
                .create_account(client_options)
                .unwrap()
                .alias("alias")
                .initialise()
                .await
                .unwrap();
        })
        .await;

        // TODO improve test when the node API is ready to use
    }

    // this needs a proper client mock to run on CI
    // #[tokio::test]
    #[allow(dead_code)]
    async fn dust_transfer() {
        let manager = crate::test_utils::get_account_manager().await;

        // first we create an address with balance - the source address
        let mut address1 = crate::test_utils::generate_random_address();
        let output = crate::address::AddressOutput {
            transaction_id: iota_client::bee_message::prelude::TransactionId::from([0; 32]),
            message_id: iota_client::bee_message::MessageId::from([0; 32]),
            index: 0,
            amount: 10000000,
            is_spent: false,
            address: address1.address().clone(),
            kind: crate::address::OutputKind::SignatureLockedSingle,
        };
        address1.outputs.insert(output.id().unwrap(), output);

        // then we create an address without balance - the deposit address
        let address2 = crate::test_utils::generate_random_address();

        let mut address3 = crate::test_utils::generate_random_address();
        address3.set_key_index(0);
        address3.set_internal(true);
        let output = crate::address::AddressOutput {
            transaction_id: iota_client::bee_message::prelude::TransactionId::from([1; 32]),
            message_id: iota_client::bee_message::MessageId::from([1; 32]),
            index: 0,
            amount: 10000000,
            is_spent: false,
            address: address3.address().clone(),
            kind: crate::address::OutputKind::SignatureLockedDustAllowance,
        };
        address3.outputs.insert(output.id().unwrap(), output);

        println!(
            "{}\n{}\n{}",
            address1.address().to_bech32(),
            address2.address().to_bech32(),
            address3.address().to_bech32()
        );

        let account_handle = crate::test_utils::AccountCreator::new(&manager)
            .addresses(vec![address1, address2.clone(), address3])
            .create()
            .await;
        let id = account_handle.id().await;
        let index = account_handle.index().await;
        let synced = super::SyncedAccount {
            id,
            index,
            account_handle,
            deposit_address: crate::test_utils::generate_random_address(),
            is_empty: false,
            messages: Vec::new(),
            addresses: Vec::new(),
        };
        let res = synced
            .transfer(
                super::Transfer::builder(
                    address2.address().clone(),
                    std::num::NonZeroU64::new(999500).unwrap(),
                    None,
                )
                .finish(),
            )
            .await;
        assert!(res.is_err());
        match res.unwrap_err() {
            crate::Error::DustError(_) => {}
            _ => panic!("unexpected response"),
        }
    }

    fn _generate_address_output(amount: u64, is_spent: bool) -> AddressOutput {
        let mut tx_id = [0; 32];
        crypto::utils::rand::fill(&mut tx_id).unwrap();
        AddressOutput {
            transaction_id: TransactionId::new(tx_id),
            message_id: MessageId::new([0; 32]),
            index: 0,
            amount,
            is_spent,
            address: crate::test_utils::generate_random_iota_address(),
            kind: OutputKind::SignatureLockedSingle,
        }
    }

    #[quickcheck]
    fn balance_change_event(old_balance: u32, new_balance: u32, outputs: Vec<(u64, bool)>) {
        let address = crate::test_utils::generate_random_iota_address();
        let mut address_outputs = HashMap::new();
        for (amount, is_spent) in outputs {
            let output = _generate_address_output(amount, is_spent);
            address_outputs.insert(output.id().unwrap(), output);
        }
        let events = super::get_balance_change_events(
            old_balance.into(),
            new_balance.into(),
            address,
            Default::default(),
            Default::default(),
            &address_outputs,
        );
        assert_eq!(
            new_balance as i64,
            old_balance as i64
                + events.iter().fold(0i64, |a, c| a - (c.balance_change.spent as i64)
                    + (c.balance_change.received as i64))
        );
    }
}
