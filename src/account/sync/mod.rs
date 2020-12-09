// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountIdentifier},
    address::{Address, AddressBuilder, AddressOutput, IotaAddress},
    client::get_client,
    message::{Message, RemainderValueStrategy, Transfer},
};

use getset::Getters;
use iota::message::prelude::{
    Input, Message as IotaMessage, MessageId, Payload, SignatureLockedSingleOutput, Transaction, TransactionEssence,
    TransactionId, UTXOInput,
};
use serde::{Deserialize, Serialize};
use slip10::BIP32Path;

use std::{convert::TryInto, num::NonZeroU64, path::PathBuf};

mod input_selection;

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
    storage_path: &PathBuf,
    account: &'_ Account,
    address_index: usize,
    gap_limit: usize,
) -> crate::Result<(Vec<Address>, Vec<(MessageId, IotaMessage)>)> {
    let mut address_index = address_index;

    let client = crate::client::get_client(account.client_options());
    let client = client.read().unwrap();

    let mut generated_addresses = vec![];
    let mut found_messages = vec![];
    loop {
        let mut generated_iota_addresses = vec![]; // collection of (address_index, internal, address) pairs
        for i in address_index..(address_index + gap_limit) {
            // generate both `public` and `internal (change)` addresses
            generated_iota_addresses.push((i, false, crate::address::get_iota_address(&account, i, false)?));
            generated_iota_addresses.push((i, true, crate::address::get_iota_address(&account, i, true)?));
        }

        let mut curr_generated_addresses = vec![];
        let mut curr_found_messages = vec![];

        for (iota_address_index, iota_address_internal, iota_address) in &generated_iota_addresses {
            let address_outputs = client.get_address().outputs(&iota_address).await?;
            let balance = client.get_address().balance(&iota_address).await?;

            log::debug!(
                "[SYNC] syncing address {}, got {} outputs and balance {}",
                iota_address.to_bech32(),
                address_outputs.len(),
                balance
            );

            let mut curr_found_outputs: Vec<AddressOutput> = vec![];
            for output in address_outputs.iter() {
                let output = client.get_output(output).await?;
                if let Ok(message) = client
                    .get_message()
                    .data(&MessageId::new(
                        output.message_id[..]
                            .try_into()
                            .map_err(|_| crate::WalletError::InvalidMessageIdLength)?,
                    ))
                    .await
                {
                    curr_found_messages.push((
                        MessageId::new(
                            output.message_id[..]
                                .try_into()
                                .map_err(|_| crate::WalletError::InvalidMessageIdLength)?,
                        ),
                        message,
                    ));
                }
                let output = output.try_into()?;
                log::debug!("[SYNC] found output {:?}", output);
                curr_found_outputs.push(output);
            }

            // ignore unused change addresses
            if *iota_address_internal && curr_found_outputs.is_empty() {
                log::debug!("[SYNC] ignoring address because it's internal and the output list is empty");
                continue;
            }

            let address = AddressBuilder::new()
                .address(iota_address.clone())
                .key_index(*iota_address_index)
                .balance(balance)
                .outputs(curr_found_outputs)
                .internal(*iota_address_internal)
                .build()?;

            curr_generated_addresses.push(address);
        }

        address_index += gap_limit;

        let is_empty = curr_found_messages.is_empty()
            && curr_generated_addresses
                .iter()
                .all(|address| !address.outputs().iter().any(|output| *output.is_spent()));

        found_messages.extend(curr_found_messages.into_iter());
        generated_addresses.extend(curr_generated_addresses.into_iter());

        if is_empty {
            log::debug!("[SYNC] finishing address syncing because the current messages list and address list is empty");
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
) -> crate::Result<Vec<(MessageId, IotaMessage)>> {
    let mut messages = vec![];
    let client = crate::client::get_client(account.client_options());
    let client = client.read().unwrap();

    for address in account.addresses_mut().iter_mut().take(stop_at_address_index) {
        log::debug!(
            "[SYNC] syncing messages and outputs for address {}",
            address.address().to_bech32()
        );
        let address_outputs = client.get_address().outputs(address.address()).await?;
        let balance = client.get_address().balance(address.address()).await?;

        log::debug!("[SYNC] got {} outputs and balance {}", address_outputs.len(), balance);

        let mut outputs = vec![];
        for output in address_outputs.iter() {
            let output = client.get_output(output).await?;
            let output: AddressOutput = output.try_into()?;

            if let Ok(message) = client.get_message().data(output.message_id()).await {
                messages.push((*output.message_id(), message));
            }

            log::debug!("[SYNC] found output {:?}", output);

            outputs.push(output);
        }

        address.set_outputs(outputs);
        address.set_balance(balance);
    }

    Ok(messages)
}

async fn update_account_messages<'a>(
    account: &'a Account,
    new_messages: &'a [(MessageId, IotaMessage)],
) -> crate::Result<()> {
    let mut messages: Vec<Message> = account.messages().to_vec();

    // sync `broadcasted` state
    messages
        .iter_mut()
        .filter(|message| !message.broadcasted() && new_messages.iter().any(|(id, _)| id == message.id()))
        .for_each(|message| {
            log::debug!("[SYNC] marking message {:?} as broadcasted", message.id());
            message.set_broadcasted(true);
        });

    // sync `confirmed` state
    let mut unconfirmed_messages: Vec<&mut Message> =
        messages.iter_mut().filter(|message| !message.confirmed()).collect();
    let client = get_client(account.client_options());
    let client = client.read().unwrap();
    for message in unconfirmed_messages.iter_mut() {
        let metadata = client.get_message().metadata(message.id()).await?;
        let confirmed = !(metadata.should_promote.unwrap_or(true) || metadata.should_reattach.unwrap_or(true));
        if confirmed {
            log::debug!("[SYNC] marking message {:?} as confirmed", message.id());
            message.set_confirmed(true);
        }
    }

    Ok(())
}

async fn perform_sync(
    mut account: &mut Account,
    storage_path: &PathBuf,
    address_index: usize,
    gap_limit: usize,
) -> crate::Result<bool> {
    log::debug!(
        "[SYNC] syncing with address_index = {}, gap_limit = {}",
        address_index,
        gap_limit
    );
    let (found_addresses, found_messages) = sync_addresses(&storage_path, &account, address_index, gap_limit).await?;

    let mut new_messages = vec![];
    for (found_message_id, found_message) in found_messages {
        if !account
            .messages()
            .iter()
            .any(|message| message.id() == &found_message_id)
        {
            new_messages.push((found_message_id, found_message));
        }
    }

    let synced_messages = sync_messages(&mut account, address_index).await?;
    new_messages.extend(synced_messages.into_iter());

    update_account_messages(&account, &new_messages).await?;

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
        .iter()
        .map(|(id, message)| Message::from_iota_message(*id, account.addresses(), &message).unwrap())
        .collect();
    log::debug!("[SYNC] new messages: {:#?}", parsed_messages);
    account.append_messages(parsed_messages);

    log::debug!("[SYNC] is empty: {}", is_empty);

    Ok(is_empty)
}

/// Account sync helper.
pub struct AccountSynchronizer<'a> {
    account: &'a mut Account,
    address_index: usize,
    gap_limit: usize,
    skip_persistance: bool,
    storage_path: PathBuf,
}

impl<'a> AccountSynchronizer<'a> {
    /// Initialises a new instance of the sync helper.
    pub(super) fn new(account: &'a mut Account, storage_path: PathBuf) -> Self {
        let address_index = account.addresses().len();
        Self {
            account,
            // by default we synchronize from the latest address (supposedly unspent)
            address_index: if address_index == 0 { 0 } else { address_index - 1 },
            gap_limit: if address_index == 0 { 10 } else { 1 },
            skip_persistance: false,
            storage_path,
        }
    }

    /// Number of address indexes that are generated.
    pub fn gap_limit(mut self, limit: usize) -> Self {
        self.gap_limit = limit;
        self
    }

    /// Skip write to the database.
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    /// Initial address index to start syncing.
    pub fn address_index(mut self, address_index: usize) -> Self {
        self.address_index = address_index;
        self
    }

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    pub async fn execute(self) -> crate::Result<SyncedAccount> {
        let options = self.account.client_options().clone();
        let client = get_client(&options);

        if let Err(e) = crate::monitor::unsubscribe(&self.account) {
            log::error!("[MQTT] error unsubscribing from MQTT topics before syncing: {:?}", e);
        }

        let mut account_ = self.account.clone();
        let return_value =
            match perform_sync(&mut account_, &self.storage_path, self.address_index, self.gap_limit).await {
                Ok(is_empty) => {
                    self.account.set_addresses(account_.addresses().to_vec());
                    self.account.set_messages(account_.messages().to_vec());
                    if !self.skip_persistance {
                        crate::storage::with_adapter(&self.storage_path, |storage| {
                            storage.set(self.account.id().into(), serde_json::to_string(&self.account)?)
                        })?;
                    }

                    let synced_account = SyncedAccount {
                        account_id: self.account.id().clone(),
                        deposit_address: self.account.latest_address().unwrap().clone(),
                        is_empty,
                        storage_path: self.storage_path,
                        addresses: self.account.addresses().clone(),
                        messages: self.account.messages().clone(),
                    };
                    Ok(synced_account)
                }
                Err(e) => Err(e),
            };

        if let Err(e) = crate::monitor::monitor_account_addresses_balance(&self.account) {
            log::error!("[MQTT] error reinitialising account addresses monitoring: {:?}", e);
        }
        if let Err(e) = crate::monitor::monitor_unconfirmed_messages(&self.account) {
            log::error!(
                "[MQTT] error reinitialising account unconfirmed messages monitoring: {:?}",
                e
            );
        }

        return_value
    }
}

/// Data returned from account synchronization.
#[derive(Debug, Clone, PartialEq, Getters, Serialize, Deserialize)]
pub struct SyncedAccount {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    #[getset(get = "pub")]
    account_id: String,
    /// The account's deposit address.
    #[serde(rename = "depositAddress")]
    #[getset(get = "pub")]
    deposit_address: Address,
    /// Whether the synced account is empty or not.
    #[serde(rename = "isEmpty")]
    #[getset(get = "pub(crate)")]
    is_empty: bool,
    /// The account messages.
    #[getset(get = "pub")]
    messages: Vec<Message>,
    /// The account addresses.
    #[getset(get = "pub")]
    addresses: Vec<Address>,
    #[serde(rename = "storagePath")]
    storage_path: PathBuf,
}

/// Transfer response metadata.
pub struct TransferMetadata {
    /// The transfer message.
    pub message: Message,
    /// The transfer source account with new message and addresses attached.
    pub account: Account,
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
        threshold: u64,
        account: &'a Account,
        address: &'a IotaAddress,
    ) -> crate::Result<(Vec<Address>, Option<Address>)> {
        let mut available_addresses: Vec<Address> = account
            .addresses()
            .iter()
            .cloned()
            .filter(|a| a.address() != address && *a.balance() > 0)
            .collect();
        let addresses = input_selection::select_input(threshold, &mut available_addresses)?;
        let remainder = if addresses.iter().fold(0, |acc, a| acc + a.balance()) > threshold {
            addresses.last().cloned()
        } else {
            None
        };
        Ok((addresses, remainder))
    }

    /// Send messages.
    pub async fn transfer(&self, transfer_obj: Transfer) -> crate::Result<TransferMetadata> {
        log::debug!("[TRANFER] got transfer {:?}", transfer_obj);

        // validate the transfer
        if transfer_obj.amount == 0 {
            return Err(crate::WalletError::ZeroAmount);
        }

        // prepare the transfer getting some needed objects and values
        let value: u64 = transfer_obj.amount;
        let account_id: AccountIdentifier = self.account_id.clone().into();
        let mut account = crate::storage::get_account(&self.storage_path, account_id.clone())?;
        let mut addresses_to_watch = vec![];

        let client = crate::client::get_client(account.client_options());
        let client = client.read().unwrap();

        if let RemainderValueStrategy::AccountAddress(ref remainder_target_address) =
            transfer_obj.remainder_value_strategy
        {
            if !account
                .addresses()
                .iter()
                .any(|addr| addr.address() == remainder_target_address)
            {
                return Err(crate::WalletError::InvalidRemainderValueAddress);
            }
        }

        // select the input addresses and check if a remainder address is needed
        let (input_addresses, remainder_address) =
            self.select_inputs(transfer_obj.amount, &account, &transfer_obj.address)?;

        log::debug!(
            "[TRANSFER] inputs: {:#?} - remainder address: {:?}",
            input_addresses,
            remainder_address
        );

        let mut utxos = vec![];
        let mut address_index_recorders = vec![];

        for input_address in &input_addresses {
            let address_outputs = input_address.outputs();
            let mut outputs = vec![];
            let address_path = BIP32Path::from_str(&format!(
                "m/44H/4218H/{}H/{}H/{}H",
                account.index(),
                *input_address.internal() as u32,
                *input_address.key_index()
            ))
            .unwrap();
            for (offset, output) in address_outputs.iter().enumerate() {
                let output = client
                    .get_output(
                        &UTXOInput::new(*output.transaction_id(), *output.index())
                            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
                    )
                    .await?;
                outputs.push((output, *input_address.key_index(), address_path.clone()));
            }
            utxos.extend(outputs.into_iter());
        }

        let mut essence_builder = TransactionEssence::builder();
        let mut current_output_sum = 0;
        let mut remainder_value = 0;
        for (utxo, address_index, address_path) in utxos {
            let input: Input = UTXOInput::new(
                TransactionId::new(
                    utxo.transaction_id[..]
                        .try_into()
                        .map_err(|_| crate::WalletError::InvalidTransactionIdLength)?,
                ),
                utxo.output_index,
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .into();
            essence_builder = essence_builder.add_input(input.clone());
            address_index_recorders.push(crate::signing::TransactionInput {
                input,
                address_index,
                address_path,
            });
            if current_output_sum == value {
                log::debug!(
                    "[TRANFER] current output sum matches the transfer value, adding {} to the remainder value (currently at {})",
                    utxo.amount,
                    remainder_value
                );
                // already filled the transfer value; just collect the output value as remainder
                remainder_value += utxo.amount;
            } else if current_output_sum + utxo.amount > value {
                log::debug!(
                    "[TRANFER] current output sum ({}) would exceed the transfer value if added the output amount ({})",
                    current_output_sum,
                    utxo.amount
                );
                // if the used UTXO amount is greater than the transfer value, this is the last iteration and we'll have
                // remainder value. we add an Output for the missing value and collect the remainder
                let missing_value = value - current_output_sum;
                remainder_value += utxo.amount - missing_value;
                essence_builder = essence_builder.add_output(
                    SignatureLockedSingleOutput::new(
                        transfer_obj.address.clone(),
                        NonZeroU64::new(missing_value).ok_or_else(|| anyhow::anyhow!("invalid amount"))?,
                    )
                    .into(),
                );
                current_output_sum += missing_value;
                log::debug!(
                    "[TRANSFER] added output with the missing value {}, and the remainder is {}",
                    missing_value,
                    remainder_value
                );
            } else {
                log::debug!(
                    "[TRANSFER] adding output amount {}, current sum {}",
                    utxo.amount,
                    current_output_sum
                );
                essence_builder = essence_builder.add_output(
                    SignatureLockedSingleOutput::new(
                        transfer_obj.address.clone(),
                        NonZeroU64::new(utxo.amount).ok_or_else(|| anyhow::anyhow!("invalid amount"))?,
                    )
                    .into(),
                );
                current_output_sum += utxo.amount;
            }
        }

        // if there's remainder value, we check the strategy defined in the transfer
        let mut remainder_value_deposit_address = None;
        if remainder_value > 0 {
            let remainder_address =
                remainder_address.ok_or_else(|| anyhow::anyhow!("remainder address not defined"))?;

            println!("[TRANFER] remainder value is {}", remainder_value);

            let remainder_target_address = match transfer_obj.remainder_value_strategy {
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
                        let deposit_address = account.latest_address().unwrap().address().clone();
                        log::debug!("[TRANFER] the remainder address is internal, so using latest address as remainder target: {}", deposit_address.to_bech32());
                        deposit_address
                    } else {
                        let change_address = crate::address::get_new_change_address(&account, &remainder_address)?;
                        let addr = change_address.address().clone();
                        log::debug!(
                            "[TRANSFER] generated new change address as remainder target: {}",
                            addr.to_bech32()
                        );
                        account.append_addresses(vec![change_address]);
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
            remainder_value_deposit_address = Some(remainder_target_address.clone());
            essence_builder = essence_builder.add_output(
                SignatureLockedSingleOutput::new(
                    remainder_target_address,
                    NonZeroU64::new(remainder_value).ok_or_else(|| anyhow::anyhow!("invalid amount"))?,
                )
                .into(),
            );
        }

        let (parent1, parent2) = client.get_tips().await?;

        let essence = essence_builder
            .finish()
            .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

        let unlock_blocks = crate::signing::with_signer(account.signer_type(), |signer| {
            signer.sign_message(&account, &essence, &mut address_index_recorders)
        })?;
        let mut tx_builder = Transaction::builder().with_essence(essence);
        for unlock_block in unlock_blocks {
            tx_builder = tx_builder.add_unlock_block(unlock_block);
        }
        let transaction = tx_builder.finish().map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

        let message = IotaMessage::builder()
            .with_parent1(parent1)
            .with_parent2(parent2)
            .with_payload(Payload::Transaction(Box::new(transaction)))
            .with_network_id(0)
            .finish()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        log::debug!("[TRANSFER] submitting message {:#?}", message);

        let message_id = client.post_message(&message).await?;
        // if this is a transfer to the account's latest address or we used the latest as deposit of the remainder
        // value, we generate a new one to keep the latest address unused
        let latest_address = account.latest_address().unwrap().address();
        if latest_address == &transfer_obj.address
            || (remainder_value_deposit_address.is_some()
                && &remainder_value_deposit_address.unwrap() == latest_address)
        {
            log::debug!(
                "[TRANSFER] generating new address since {}",
                if latest_address == &transfer_obj.address {
                    "latest address equals the transfer address"
                } else {
                    "latest address equals the remainder value deposit address"
                }
            );
            let addr = crate::address::get_new_address(&account)?;
            addresses_to_watch.push(addr.address().clone());
            account.append_addresses(vec![addr]);
        }

        let message = client.get_message().data(&message_id).await?;

        // drop the client ref so it doesn't lock the monitor system
        std::mem::drop(client);

        for address in addresses_to_watch {
            // ignore errors because we fallback to the polling system
            if let Err(e) = crate::monitor::monitor_address_balance(&account, &address) {
                log::error!("[MQTT] error monitoring new account address: {:?}", e);
            }
        }

        let message = Message::from_iota_message(message_id, account.addresses(), &message)?;
        account.append_messages(vec![message.clone()]);
        crate::storage::with_adapter(&self.storage_path, |storage| {
            storage.set(account_id, serde_json::to_string(&account)?)
        })?;

        // ignore errors because we fallback to the polling system
        if let Err(e) = crate::monitor::monitor_confirmation_state_change(&account, &message_id) {
            log::error!("[MQTT] error monitoring for confirmation change: {:?}", e);
        }

        Ok(TransferMetadata { message, account })
    }

    /// Retry message.
    pub async fn retry(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(
            self.account_id.clone().into(),
            &self.storage_path,
            message_id,
            RepostAction::Retry,
        )
        .await
    }

    /// Promote message.
    pub async fn promote(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(
            self.account_id.clone().into(),
            &self.storage_path,
            message_id,
            RepostAction::Promote,
        )
        .await
    }

    /// Reattach message.
    pub async fn reattach(&self, message_id: &MessageId) -> crate::Result<Message> {
        repost_message(
            self.account_id.clone().into(),
            &self.storage_path,
            message_id,
            RepostAction::Reattach,
        )
        .await
    }
}

pub(crate) enum RepostAction {
    Retry,
    Reattach,
    Promote,
}

pub(crate) async fn repost_message(
    account_id: AccountIdentifier,
    storage_path: &PathBuf,
    message_id: &MessageId,
    action: RepostAction,
) -> crate::Result<Message> {
    let mut account: Account = crate::storage::get_account(&storage_path, account_id)?;
    match account.get_message(message_id) {
        Some(message_to_repost) => {
            // get the latest reattachment of the message we want to promote/rettry/reattach
            let messages = account.list_messages(0, 0, None);
            let message_to_repost = messages
                .iter()
                .find(|m| m.payload() == message_to_repost.payload())
                .unwrap();
            if *message_to_repost.confirmed() {
                return Err(crate::WalletError::ClientError(
                    iota::client::Error::NoNeedPromoteOrReattach(message_id.to_string()),
                ));
            }

            let client = crate::client::get_client(account.client_options());
            let client = client.read().unwrap();

            let (id, message) = match action {
                RepostAction::Promote => {
                    let metadata = client.get_message().metadata(message_id).await?;
                    if metadata.should_promote.unwrap_or(false) {
                        client.promote(message_id).await?
                    } else {
                        return Err(crate::WalletError::ClientError(
                            iota::client::Error::NoNeedPromoteOrReattach(message_id.to_string()),
                        ));
                    }
                }
                RepostAction::Reattach => {
                    let metadata = client.get_message().metadata(message_id).await?;
                    if metadata.should_reattach.unwrap_or(false) {
                        client.reattach(message_id).await?
                    } else {
                        return Err(crate::WalletError::ClientError(
                            iota::client::Error::NoNeedPromoteOrReattach(message_id.to_string()),
                        ));
                    }
                }
                RepostAction::Retry => client.retry(message_id).await?,
            };
            let message = Message::from_iota_message(id, account.addresses(), &message)?;

            account.append_messages(vec![message.clone()]);
            crate::storage::with_adapter(&storage_path, |storage| {
                storage.set(account.id().into(), serde_json::to_string(&account)?)
            })?;

            Ok(message)
        }
        None => Err(crate::WalletError::MessageNotFound),
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptionsBuilder;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn account_sync() {
            crate::block_on(async move {
                let manager = crate::test_utils::get_account_manager();
                let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                    .unwrap()
                    .build();
                let account = manager
                    .create_account(client_options)
                    .alias("alias")
                    .initialise()
                    .unwrap();

                // let synced_accounts = account.sync().execute().await.unwrap();
                // TODO improve test when the node API is ready to use
            });
        }
    }
}
