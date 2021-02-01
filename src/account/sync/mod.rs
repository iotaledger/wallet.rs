// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountHandle},
    address::{Address, AddressBuilder, AddressOutput, AddressWrapper},
    message::{Message, RemainderValueStrategy, Transfer},
    signing::{GenerateAddressMetadata, SignMessageMetadata},
};

use bee_rest_api::handlers::message_metadata::LedgerInclusionStateDto;
use getset::Getters;
use iota::{
    message::prelude::{
        Input, Message as IotaMessage, MessageBuilder, MessageId, Payload, SignatureLockedSingleOutput,
        TransactionPayload, TransactionPayloadEssence, UTXOInput,
    },
    ClientMiner,
};
use serde::Serialize;
use slip10::BIP32Path;
use tokio::{
    sync::{mpsc::channel, MutexGuard},
    time::sleep,
};

use std::{
    convert::TryInto,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

mod input_selection;

const OUTPUT_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) async fn sync_address(
    account: &Account,
    address: &mut Address,
    bech32_hrp: String,
) -> crate::Result<Vec<(MessageId, Option<bool>, IotaMessage)>> {
    let client = crate::client::get_client(account.client_options());
    let client = client.read().await;

    let iota_address = address.address();

    let address_outputs = client.get_address().outputs(&iota_address.to_bech32().into()).await?;
    let balance = client
        .get_address()
        .balance(&iota_address.to_bech32().into())
        .await?
        .balance;
    let mut found_messages = vec![];

    log::debug!(
        "[SYNC] syncing address {}, got {} outputs and balance {}",
        iota_address.to_bech32(),
        address_outputs.len(),
        balance
    );

    let mut found_outputs: Vec<AddressOutput> = vec![];
    for output in address_outputs.iter() {
        let output = client.get_output(output).await?;
        let message_id = MessageId::new(
            hex::decode(&output.message_id).map_err(|_| crate::Error::InvalidMessageId)?[..]
                .try_into()
                .map_err(|_| crate::Error::InvalidMessageIdLength)?,
        );
        found_outputs.push(AddressOutput::from_output_response(output, bech32_hrp.to_string())?);

        // if we already have the message stored
        // and the confirmation state is known
        // we skip the `get_message` call
        if account
            .messages()
            .iter()
            .any(|m| m.id() == &message_id && m.confirmed().is_some())
        {
            continue;
        }

        if let Ok(message) = client.get_message().data(&message_id).await {
            if let Ok(metadata) = client.get_message().metadata(&message_id).await {
                found_messages.push((
                    message_id,
                    metadata
                        .ledger_inclusion_state
                        .map(|l| l == LedgerInclusionStateDto::Included),
                    message,
                ));
            }
        }
    }

    address.set_balance(balance);
    address.set_outputs(found_outputs);

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
) -> crate::Result<AddressWrapper> {
    if let Some(address) = account
        .addresses()
        .iter()
        .find(|a| *a.key_index() == index && *a.internal() == internal)
    {
        Ok(address.address().clone())
    } else {
        crate::address::get_iota_address(
            &account,
            index,
            internal,
            bech32_hrp,
            GenerateAddressMetadata { syncing: true },
        )
        .await
    }
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
    account: &Account,
    address_index: usize,
    gap_limit: usize,
) -> crate::Result<(Vec<Address>, Vec<(MessageId, Option<bool>, IotaMessage)>)> {
    let mut address_index = address_index;

    let mut generated_addresses = vec![];
    let mut found_messages = vec![];

    let bech32_hrp = crate::client::get_client(account.client_options())
        .read()
        .await
        .get_network_info()
        .bech32_hrp;

    loop {
        let mut generated_iota_addresses = vec![]; // collection of (address_index, internal, address) pairs
        for i in address_index..(address_index + gap_limit) {
            // generate both `public` and `internal (change)` addresses
            generated_iota_addresses.push((
                i,
                false,
                get_address_for_sync(&account, bech32_hrp.to_string(), i, false).await?,
            ));
            generated_iota_addresses.push((
                i,
                true,
                get_address_for_sync(&account, bech32_hrp.to_string(), i, true).await?,
            ));
        }

        let mut curr_generated_addresses = vec![];
        let mut curr_found_messages = vec![];

        let mut futures_ = Vec::new();
        for (iota_address_index, iota_address_internal, iota_address) in &generated_iota_addresses {
            let bech32_hrp_ = bech32_hrp.clone();
            futures_.push(async move {
                let mut address = AddressBuilder::new()
                    .address(iota_address.clone())
                    .key_index(*iota_address_index)
                    .balance(0)
                    .outputs(Vec::new())
                    .internal(*iota_address_internal)
                    .build()?;
                let messages = sync_address(&account, &mut address, bech32_hrp_).await?;
                crate::Result::Ok((messages, address))
            });
        }

        let results = futures::future::join_all(futures_).await;
        for result in results {
            let (found_messages, address) = result?;
            // if the address is a change address and has no outputs, we ignore it
            if !(*address.internal() && address.outputs().is_empty()) {
                curr_generated_addresses.push(address);
            }
            curr_found_messages.extend(found_messages);
        }

        address_index += gap_limit;

        let is_empty = curr_found_messages.is_empty()
            && curr_generated_addresses
                .iter()
                .all(|address| !address.outputs().iter().any(|output| *output.is_spent()));

        found_messages.extend(curr_found_messages.into_iter());
        generated_addresses.extend(curr_generated_addresses.into_iter());

        if is_empty {
            log::debug!(
                "[SYNC] finishing address syncing because the current messages list and address list are empty"
            );
            break;
        }
    }

    Ok((generated_addresses, found_messages))
}

/// Syncs messages with the tangle.
/// The method should ensures that the wallet local state has messages associated with the address history.
async fn sync_messages(
    account: &mut Account,
    stop_at_address_index: usize,
) -> crate::Result<Vec<(MessageId, Option<bool>, IotaMessage)>> {
    let mut messages = vec![];
    let client_options = account.client_options().clone();

    let messages_with_known_confirmation: Vec<MessageId> = account
        .messages()
        .iter()
        .filter(|m| m.confirmed().is_some())
        .map(|m| *m.id())
        .collect();

    let futures_ = account
        .addresses_mut()
        .iter_mut()
        .filter(|address| *address.key_index() < stop_at_address_index)
        .map(|address| {
            let client_options = client_options.clone();
            let messages_with_known_confirmation = messages_with_known_confirmation.clone();
            async move {
                let client = crate::client::get_client(&client_options);
                let client = client.read().await;

                let address_outputs = client
                    .get_address()
                    .outputs(&address.address().to_bech32().into())
                    .await?;
                let balance = client
                    .get_address()
                    .balance(&address.address().to_bech32().into())
                    .await?
                    .balance;

                log::debug!(
                    "[SYNC] syncing messages and outputs for address {}, got {} outputs and balance {}",
                    address.address().to_bech32(),
                    address_outputs.len(),
                    balance
                );

                let mut outputs = vec![];
                let mut messages = vec![];
                for output in address_outputs.iter() {
                    let output = client.get_output(output).await?;
                    let output =
                        AddressOutput::from_output_response(output, address.address().bech32_hrp().to_string())?;
                    let output_message_id = *output.message_id();

                    outputs.push(output);

                    // if we already have the message stored
                    // and the confirmation state is known
                    // we skip the `get_message` call
                    if messages_with_known_confirmation.contains(&output_message_id) {
                        continue;
                    }

                    if let Ok(message) = client.get_message().data(&output_message_id).await {
                        if let Ok(metadata) = client.get_message().metadata(&output_message_id).await {
                            messages.push((
                                output_message_id,
                                metadata
                                    .ledger_inclusion_state
                                    .map(|l| l == LedgerInclusionStateDto::Included),
                                message,
                            ));
                        }
                    }
                }

                address.set_outputs(outputs);
                address.set_balance(balance);

                crate::Result::Ok(messages)
            }
        });

    for res in futures::future::join_all(futures_).await {
        messages.extend(res?);
    }

    Ok(messages)
}

async fn perform_sync(
    mut account: &mut Account,
    address_index: usize,
    gap_limit: usize,
    steps: Vec<AccountSynchronizeStep>,
) -> crate::Result<bool> {
    log::debug!(
        "[SYNC] syncing with address_index = {}, gap_limit = {}",
        address_index,
        gap_limit
    );
    let (found_addresses, found_messages) = if steps.contains(&AccountSynchronizeStep::SyncAddresses) {
        sync_addresses(&account, address_index, gap_limit).await?
    } else {
        (Vec::new(), Vec::new())
    };

    let mut new_messages = vec![];
    for (found_message_id, confirmed, found_message) in found_messages {
        if !account
            .messages()
            .iter()
            .any(|message| message.id() == &found_message_id)
        {
            new_messages.push((found_message_id, confirmed, found_message));
        }
    }

    if steps.contains(&AccountSynchronizeStep::SyncMessages) {
        let synced_messages = sync_messages(&mut account, address_index).await?;
        new_messages.extend(synced_messages.into_iter());
    }

    let mut addresses_to_save = vec![];
    let mut ignored_addresses = vec![];
    let mut previous_address_is_unused = false;
    for found_address in found_addresses.into_iter() {
        let address_is_unused = found_address.outputs().is_empty();

        // if the previous address is unused, we'll keep checking to see if an used address was found on the gap limit
        if previous_address_is_unused {
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
        // if the previous address is used or this is the first address,
        // we'll save it because we want at least one unused address
        else {
            addresses_to_save.push(found_address);
        }
        previous_address_is_unused = address_is_unused;
    }
    log::debug!("[SYNC] new addresses: {:#?}", addresses_to_save);

    let is_empty = new_messages.is_empty() && addresses_to_save.iter().all(|address| address.outputs().is_empty());

    account.append_addresses(addresses_to_save);

    let parsed_messages = new_messages
        .into_iter()
        .map(|(id, confirmed, message)| Message::from_iota_message(id, account.addresses(), message, confirmed))
        .collect();
    log::debug!("[SYNC] new messages: {:#?}", parsed_messages);
    account.append_messages(parsed_messages);

    log::debug!("[SYNC] is empty: {}", is_empty);

    Ok(is_empty)
}

#[derive(PartialEq)]
pub(crate) enum AccountSynchronizeStep {
    SyncAddresses,
    SyncMessages,
}

/// Account sync helper.
pub struct AccountSynchronizer {
    account_handle: AccountHandle,
    address_index: usize,
    gap_limit: usize,
    skip_persistance: bool,
    steps: Vec<AccountSynchronizeStep>,
}

impl AccountSynchronizer {
    /// Initialises a new instance of the sync helper.
    pub(super) async fn new(account_handle: AccountHandle) -> Self {
        let address_index = account_handle.read().await.addresses().len();
        Self {
            account_handle,
            // by default we synchronize from the latest address (supposedly unspent)
            address_index: if address_index == 0 { 0 } else { address_index - 1 },
            gap_limit: if address_index == 0 { 10 } else { 1 },
            skip_persistance: false,
            steps: vec![
                AccountSynchronizeStep::SyncAddresses,
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
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
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

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    pub async fn execute(self) -> crate::Result<SyncedAccount> {
        if let Err(e) = crate::monitor::unsubscribe(self.account_handle.clone()).await {
            log::error!("[MQTT] error unsubscribing from MQTT topics before syncing: {:?}", e);
        }

        let mut account_ = {
            let account_ref = self.account_handle.read().await;
            account_ref.clone()
        };
        let messages_before_sync: Vec<(MessageId, Option<bool>)> =
            account_.messages().iter().map(|m| (*m.id(), *m.confirmed())).collect();
        let addresses_before_sync: Vec<(String, u64, Vec<AddressOutput>)> = account_
            .addresses()
            .iter()
            .map(|a| (a.address().to_bech32(), *a.balance(), a.outputs().to_vec()))
            .collect();

        let return_value = match perform_sync(&mut account_, self.address_index, self.gap_limit, self.steps).await {
            Ok(is_empty) => {
                if !self.skip_persistance {
                    let mut account_ref = self.account_handle.write().await;
                    account_ref
                        .do_mut(|account| {
                            account.set_addresses(account_.addresses().to_vec());
                            account.set_messages(account_.messages().to_vec());
                            account.set_last_synced_at(Some(chrono::Local::now()));
                            Ok(())
                        })
                        .await?;
                }

                let account_ref = self.account_handle.read().await;

                let synced_account = SyncedAccount {
                    id: account_ref.id().to_string(),
                    index: *account_ref.index(),
                    account_handle: self.account_handle.clone(),
                    deposit_address: account_ref.latest_address().clone(),
                    is_empty,
                    addresses: account_ref
                        .addresses()
                        .iter()
                        .filter(|a| {
                            match addresses_before_sync
                                .iter()
                                .find(|(addr, _, _)| addr == &a.address().to_bech32())
                            {
                                Some((_, balance, outputs)) => balance != a.balance() || outputs != a.outputs(),
                                None => true,
                            }
                        })
                        .cloned()
                        .collect(),
                    messages: account_ref
                        .messages()
                        .iter()
                        .filter(|m| {
                            !messages_before_sync
                                .iter()
                                .any(|(id, confirmed)| id == m.id() && confirmed == m.confirmed())
                        })
                        .cloned()
                        .collect(),
                };
                Ok(synced_account)
            }
            Err(e) => Err(e),
        };

        let _ = crate::monitor::monitor_account_addresses_balance(self.account_handle.clone());
        let _ = crate::monitor::monitor_unconfirmed_messages(self.account_handle.clone());

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
    account_handle: AccountHandle,
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
    messages: Vec<Message>,
    /// The newly generated and updated account addresses.
    #[getset(get = "pub")]
    addresses: Vec<Address>,
}

impl SyncedAccount {
    /// Selects input addresses for a value transaction.
    /// The method ensures that the recipient address doesn’t match any of the selected inputs or the remainder address.
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
        locked_addresses: &'a mut MutexGuard<'_, Vec<AddressWrapper>>,
        threshold: u64,
        account: &'a Account,
        address: &'a AddressWrapper,
    ) -> crate::Result<(Vec<input_selection::Input>, Option<input_selection::Input>)> {
        let available_addresses: Vec<input_selection::Input> = account
            .addresses()
            .iter()
            .filter(|a| {
                a.address() != address && a.available_balance(&account) > 0 && !locked_addresses.contains(a.address())
            })
            .map(|a| input_selection::Input {
                address: a.address().clone(),
                internal: *a.internal(),
                balance: a.available_balance(&account),
            })
            .collect();
        let addresses = input_selection::select_input(threshold, available_addresses)?;

        locked_addresses.extend(
            addresses
                .iter()
                .map(|a| a.address.clone())
                .collect::<Vec<AddressWrapper>>(),
        );

        let remainder = if addresses.iter().fold(0, |acc, a| acc + a.balance) > threshold {
            addresses.last().cloned()
        } else {
            None
        };

        Ok((addresses, remainder))
    }

    /// Send messages.
    pub async fn transfer(&self, transfer_obj: Transfer) -> crate::Result<Message> {
        let account_ = self.account_handle.read().await;

        // lock the transfer process until we select the input addresses
        // we do this to prevent multiple threads trying to transfer at the same time
        // so it doesn't consume the same addresses multiple times, which leads to a conflict state
        let account_address_locker = self.account_handle.locked_addresses();
        let mut locked_addresses = account_address_locker.lock().await;

        // prepare the transfer getting some needed objects and values
        let value = transfer_obj.amount.get();

        let balance = account_.balance();

        if value > balance.total {
            return Err(crate::Error::InsufficientFunds);
        }

        let available_balance = balance.available;
        drop(account_);

        // if the transfer value exceeds the account's available balance,
        // wait for an account update or sync it with the tangle
        if value > available_balance {
            let (tx, mut rx) = channel(1);
            let tx = Arc::new(Mutex::new(tx));

            let account_handle = self.account_handle.clone();
            thread::spawn(move || {
                let tx = tx.lock().unwrap();
                for _ in 1..30 {
                    thread::sleep(OUTPUT_LOCK_TIMEOUT / 30);
                    let account = crate::block_on(async { account_handle.read().await });
                    // the account received an update and now the balance is sufficient
                    if value <= account.balance().available {
                        let _ = tx.send(());
                        break;
                    }
                }
            });

            let delay = sleep(Duration::from_millis(50));
            tokio::pin!(delay);
            tokio::select! {
                v = rx.recv() => {
                    if v.is_none() {
                        // if we got an error waiting for the account update, we try to sync it
                        self.account_handle.sync().await.execute().await?;
                    }
                }
                _ = &mut delay => {
                    // if we got a timeout waiting for the account update, we try to sync it
                    self.account_handle.sync().await.execute().await?;
                }
            }
        }

        let account_ = self.account_handle.read().await;

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

        // select the input addresses and check if a remainder address is needed
        let (input_addresses, remainder_address) =
            self.select_inputs(&mut locked_addresses, value, &account_, &transfer_obj.address)?;

        // unlock the transfer process since we already selected the input addresses and locked them
        drop(locked_addresses);
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

        let mut locked_addresses = account_address_locker.lock().await;
        for input_address in &input_addresses {
            let index = locked_addresses
                .iter()
                .position(|a| &input_address.address == a)
                .unwrap();
            locked_addresses.remove(index);
        }

        res
    }

    /// Retry message.
    pub async fn retry(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Retry).await
    }

    /// Promote message.
    pub async fn promote(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Promote).await
    }

    /// Reattach message.
    pub async fn reattach(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(self.account_handle.clone(), message_id, RepostAction::Reattach).await
    }
}

async fn perform_transfer(
    transfer_obj: Transfer,
    input_addresses: &[input_selection::Input],
    account_handle: AccountHandle,
    remainder_address: Option<input_selection::Input>,
) -> crate::Result<Message> {
    let mut utxos = vec![];
    let mut transaction_inputs = vec![];

    let account_ = account_handle.read().await;

    for input_address in input_addresses {
        let account_address = account_
            .addresses()
            .iter()
            .find(|a| a.address() == &input_address.address)
            .unwrap();

        let mut outputs = vec![];
        let address_path = BIP32Path::from_str(&format!(
            "m/44H/4218H/{}H/{}H/{}H",
            *account_.index(),
            *account_address.internal() as u32,
            *account_address.key_index()
        ))
        .unwrap();

        for address_output in account_address.available_outputs(&account_) {
            outputs.push((
                (*address_output).clone(),
                *account_address.key_index(),
                *account_address.internal(),
                address_path.clone(),
            ));
        }
        utxos.extend(outputs.into_iter());
    }

    let mut essence_builder = TransactionPayloadEssence::builder().add_output(
        SignatureLockedSingleOutput::new(*transfer_obj.address.as_ref(), transfer_obj.amount.get())?.into(),
    );
    let mut current_output_sum = 0;
    let mut remainder_value = 0;

    for (utxo, address_index, address_internal, address_path) in utxos {
        let input: Input = UTXOInput::new(*utxo.transaction_id(), *utxo.index())?.into();
        essence_builder = essence_builder.add_input(input.clone());
        transaction_inputs.push(crate::signing::TransactionInput {
            input,
            address_index,
            address_path,
            address_internal,
        });
        if current_output_sum == transfer_obj.amount.get() {
            log::debug!(
                    "[TRANSFER] current output sum matches the transfer value, adding {} to the remainder value (currently at {})",
                    utxo.amount(),
                    remainder_value
                );
            // already filled the transfer value; just collect the output value as remainder
            remainder_value += *utxo.amount();
        } else if current_output_sum + *utxo.amount() > transfer_obj.amount.get() {
            log::debug!(
                "[TRANSFER] current output sum ({}) would exceed the transfer value if added to the output amount ({})",
                current_output_sum,
                utxo.amount()
            );
            // if the used UTXO amount is greater than the transfer value,
            // this is the last iteration and we'll have remainder value
            let missing_value = transfer_obj.amount.get() - current_output_sum;
            remainder_value += *utxo.amount() - missing_value;
            current_output_sum += missing_value;
            log::debug!(
                "[TRANSFER] added output with the missing value {}, and the remainder is {}",
                missing_value,
                remainder_value
            );
        } else {
            log::debug!(
                "[TRANSFER] adding output amount {}, current sum {}",
                utxo.amount(),
                current_output_sum
            );
            current_output_sum += *utxo.amount();
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

        let remainder_deposit_address = match transfer_obj.remainder_value_strategy {
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
                if *remainder_address.internal() {
                    let mut deposit_address = account_.latest_address().address().clone();
                    // if the latest address is the transfer's address, we'll generate a new one as remainder deposit
                    if deposit_address == transfer_obj.address {
                        account_handle.generate_address_internal(&mut account_).await?;
                        deposit_address = account_.latest_address().address().clone();
                    }
                    log::debug!(
                        "[TRANSFER] the remainder address is internal, so using latest address as remainder target: {}",
                        deposit_address.to_bech32()
                    );
                    deposit_address
                } else {
                    let change_address = crate::address::get_new_change_address(
                        &account_,
                        &remainder_address,
                        GenerateAddressMetadata { syncing: false },
                    )
                    .await?;
                    let addr = change_address.address().clone();
                    log::debug!(
                        "[TRANSFER] generated new change address as remainder target: {}",
                        addr.to_bech32()
                    );
                    account_.append_addresses(vec![change_address]);
                    addresses_to_watch.push(addr.clone());
                    addr
                }
            }
            // keep the remainder value on the address
            RemainderValueStrategy::ReuseAddress => {
                let address = remainder_address.address().clone();
                log::debug!("[TRANSFER] reusing address as remainder target {}", address.to_bech32());
                address
            }
        };
        remainder_value_deposit_address = Some(remainder_deposit_address.clone());
        essence_builder = essence_builder
            .add_output(SignatureLockedSingleOutput::new(*remainder_deposit_address.as_ref(), remainder_value)?.into());
        Some(remainder_deposit_address)
    } else {
        None
    };

    let client = crate::client::get_client(account_.client_options());
    let client = client.read().await;

    let parents = client.get_tips().await?;

    if let Some(indexation) = transfer_obj.indexation {
        essence_builder = essence_builder.with_payload(Payload::Indexation(Box::new(indexation)));
    }

    let essence = essence_builder.finish()?;

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
            },
        )
        .await?;

    let mut tx_builder = TransactionPayload::builder().with_essence(essence);
    for unlock_block in unlock_blocks {
        tx_builder = tx_builder.add_unlock_block(unlock_block);
    }
    let transaction = tx_builder.finish()?;

    let message = MessageBuilder::<ClientMiner>::new()
        .with_parents(parents)
        .with_payload(Payload::Transaction(Box::new(transaction)))
        .with_network_id(client.get_network_id().await?)
        .with_nonce_provider(client.get_pow_provider(), client.get_network_info().min_pow_score)
        .finish()?;

    log::debug!("[TRANSFER] submitting message {:#?}", message);

    let message_id = client.post_message(&message).await?;

    // if this is a transfer to the account's latest address or we used the latest as deposit of the remainder
    // value, we generate a new one to keep the latest address unused
    let latest_address = account_.latest_address().address();
    if latest_address == &transfer_obj.address
        || (remainder_value_deposit_address.is_some() && &remainder_value_deposit_address.unwrap() == latest_address)
    {
        log::debug!(
            "[TRANSFER] generating new address since {}",
            if latest_address == &transfer_obj.address {
                "latest address equals the transfer address"
            } else {
                "latest address equals the remainder value deposit address"
            }
        );
        let addr = crate::address::get_new_address(&account_, GenerateAddressMetadata { syncing: false }).await?;
        addresses_to_watch.push(addr.address().clone());
        account_.append_addresses(vec![addr]);
    }

    let message = Message::from_iota_message(message_id, account_.addresses(), message, None);
    account_.append_messages(vec![message.clone()]);

    account_.save().await?;

    // drop the client and account_ refs so it doesn't lock the monitor system
    drop(account_);
    drop(client);

    for address in addresses_to_watch {
        // ignore errors because we fallback to the polling system
        let _ = crate::monitor::monitor_address_balance(account_handle.clone(), &address);
    }

    // ignore errors because we fallback to the polling system
    if let Err(e) = crate::monitor::monitor_confirmation_state_change(account_handle.clone(), &message_id).await {
        log::error!("[MQTT] error monitoring for confirmation change: {:?}", e);
    }

    Ok(message)
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

    let message = match account.get_message(message_id) {
        Some(message_to_repost) => {
            // get the latest reattachment of the message we want to promote/rettry/reattach
            let messages = account.list_messages(0, 0, None);
            let message_to_repost = messages
                .iter()
                .find(|m| m.payload() == message_to_repost.payload())
                .unwrap();
            if message_to_repost.confirmed().unwrap_or(false) {
                return Err(crate::Error::ClientError(iota::client::Error::NoNeedPromoteOrReattach(
                    message_id.to_string(),
                )));
            }

            let client = crate::client::get_client(account.client_options());
            let client = client.read().await;

            let (id, message) = match action {
                RepostAction::Promote => client.promote(message_id).await?,
                RepostAction::Reattach => client.reattach(message_id).await?,
                RepostAction::Retry => client.retry(message_id).await?,
            };
            let message = Message::from_iota_message(id, account.addresses(), message, None);

            account.append_messages(vec![message.clone()]);

            Ok(message)
        }
        None => Err(crate::Error::MessageNotFound),
    }?;

    Ok(message)
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptionsBuilder;

    #[tokio::test]
    async fn account_sync() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Storage, |manager, _| async move {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
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
}
