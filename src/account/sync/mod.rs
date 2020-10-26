use crate::account::{Account, AccountIdentifier};
use crate::address::{Address, AddressBuilder};
use crate::client::get_client;
use crate::message::{Message, Transfer};

use getset::Getters;
use iota::{
    client::OutputMetadata,
    message::prelude::{
        Input, Message as IotaMessage, MessageId, Output, Payload, SignatureLockedSingleOutput,
        Transaction, TransactionEssence, TransactionId, UTXOInput,
    },
};
use serde::Serialize;

use std::convert::TryInto;
use std::num::NonZeroU64;
use std::path::PathBuf;

mod input_selection;

/// Syncs addresses with the tangle.
/// The method ensures that the wallet local state has all used addresses plus an unused address.
///
/// To sync addresses for an account from scratch, `address_index` = 0 and `gap_limit` = 20 should be provided.
/// To sync addresses from the latest address, `address_index` = latest address index and `gap_limit` = 1 should be provided.
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
///
async fn sync_addresses(
    storage_path: &PathBuf,
    account: &'_ Account,
    address_index: usize,
    gap_limit: Option<usize>,
) -> crate::Result<(
    Vec<Address>,
    Vec<OutputMetadata>,
    Vec<(MessageId, IotaMessage)>,
)> {
    let mut address_index = address_index;
    let account_index = account.index()?;

    let client = get_client(account.client_options());
    let gap_limit = gap_limit.unwrap_or(20);

    let mut generated_addresses = vec![];
    let mut found_messages = vec![];
    let mut found_outputs = vec![];
    loop {
        let mut generated_iota_addresses = vec![];
        for i in address_index..(address_index + gap_limit) {
            // generate both `public` and `internal (change)` addresses
            generated_iota_addresses.push(crate::address::get_iota_address(
                &storage_path,
                account.id(),
                account_index,
                i,
                false,
            )?);
            generated_iota_addresses.push(crate::address::get_iota_address(
                &storage_path,
                account.id(),
                account_index,
                i,
                true,
            )?);
        }

        let mut curr_found_messages = vec![];
        let mut curr_found_outputs = vec![];
        for address in &generated_iota_addresses {
            let address_outputs = client.get_address().outputs(&address).await?;
            for output in address_outputs.iter() {
                let output = client.get_output(output).await?;
                let message = client
                    .get_message()
                    .data(&MessageId::new(
                        output
                            .message_id
                            .as_bytes()
                            .try_into()
                            .map_err(|_| crate::WalletError::InvalidMessageIdLength)?,
                    ))
                    .await?;
                curr_found_messages.push((
                    MessageId::new(
                        output
                            .message_id
                            .as_bytes()
                            .try_into()
                            .map_err(|_| crate::WalletError::InvalidMessageIdLength)?,
                    ),
                    message,
                ));
                curr_found_outputs.push(output);
            }
        }

        let is_spent = curr_found_outputs.iter().any(|output| output.is_spent);

        found_messages.extend(curr_found_messages.into_iter());
        found_outputs.extend(curr_found_outputs.into_iter());

        for iota_address in generated_iota_addresses {
            let balance = client.get_address().balance(&iota_address).await?;
            let address = AddressBuilder::new()
                .address(iota_address.clone())
                .key_index(address_index)
                .balance(balance)
                .build()?;
            generated_addresses.push(address);
            address_index += 1;
        }

        if found_messages.is_empty()
            && !is_spent
            && generated_addresses.iter().all(|o| *o.balance() == 0)
        {
            break;
        }
    }

    Ok((generated_addresses, found_outputs, found_messages))
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
async fn sync_transactions<'a>(
    account: &'a Account,
    new_messages: &'a [(MessageId, IotaMessage)],
) -> crate::Result<()> {
    let mut messages: Vec<Message> = account.messages().to_vec();

    // sync `broadcasted` state
    messages
        .iter_mut()
        .filter(|message| {
            !message.broadcasted() && new_messages.iter().any(|(id, _)| id == message.id())
        })
        .for_each(|message| {
            message.set_broadcasted(true);
        });

    // sync `confirmed` state
    let mut unconfirmed_messages: Vec<&mut Message> = messages
        .iter_mut()
        .filter(|message| !message.confirmed())
        .collect();
    let client = get_client(account.client_options());
    let unconfirmed_transaction_ids: Vec<MessageId> = unconfirmed_messages
        .iter()
        .map(|message| *message.id())
        .collect();
    let confirmed_states = client.is_confirmed(&unconfirmed_transaction_ids[..])?;
    for (message, confirmed) in unconfirmed_messages
        .iter_mut()
        .zip(confirmed_states.values())
    {
        if *confirmed {
            message.set_confirmed(true);
        }
    }

    Ok(())
}

/// Account sync helper.
pub struct AccountSynchronizer<'a> {
    account: &'a mut Account,
    address_index: usize,
    gap_limit: Option<usize>,
    skip_persistance: bool,
    storage_path: PathBuf,
}

impl<'a> AccountSynchronizer<'a> {
    /// Initialises a new instance of the sync helper.
    pub(super) fn new(account: &'a mut Account, storage_path: PathBuf) -> Self {
        let address_index = account.addresses().len();
        Self {
            account,
            address_index,
            gap_limit: None,
            skip_persistance: false,
            storage_path,
        }
    }

    /// Number of address indexes that are generated.
    pub fn gap_limit(mut self, limit: usize) -> Self {
        self.gap_limit = Some(limit);
        self
    }

    /// Skip write to the database.
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    pub async fn execute(self) -> crate::Result<SyncedAccount> {
        let client = get_client(self.account.client_options());
        let mut addresses = vec![];
        for address in self.account.addresses() {
            addresses.push(address.address().clone());
        }

        let (found_addresses, found_outputs, found_messages) = sync_addresses(
            &self.storage_path,
            self.account,
            self.address_index,
            self.gap_limit,
        )
        .await?;
        let is_empty = found_messages.is_empty()
            && !found_outputs.iter().any(|output| output.is_spent)
            && found_addresses.iter().all(|o| *o.balance() == 0);

        let mut new_messages = vec![];
        for (found_message_id, found_message) in found_messages {
            if !self
                .account
                .messages()
                .iter()
                .any(|message| message.id() == &found_message_id)
            {
                new_messages.push((found_message_id, found_message));
            }
        }

        sync_transactions(self.account, &new_messages).await?;

        self.account.set_messages(
            new_messages
                .iter()
                .map(|(id, message)| Message::from_iota_message(*id, &message).unwrap())
                .collect(),
        );
        self.account.set_addresses(found_addresses);

        if !self.skip_persistance {
            crate::storage::with_adapter(&self.storage_path, |storage| {
                storage.set(
                    self.account.id().into(),
                    serde_json::to_string(&self.account)?,
                )
            })?;
        }

        let synced_account = SyncedAccount {
            account_id: *self.account.id(),
            deposit_address: self.account.latest_address().unwrap().clone(),
            is_empty,
            storage_path: self.storage_path,
        };
        Ok(synced_account)
    }
}

/// Data returned from account synchronization.
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
pub struct SyncedAccount {
    /// The associated account identifier.
    #[serde(rename = "accountId")]
    #[getset(get = "pub")]
    account_id: [u8; 32],
    /// The account's deposit address.
    #[serde(rename = "depositAddress")]
    #[getset(get = "pub")]
    deposit_address: Address,
    /// Whether the synced account is empty or not.
    #[serde(rename = "isEmpty")]
    #[getset(get = "pub(crate)")]
    is_empty: bool,
    #[serde(skip)]
    storage_path: PathBuf,
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
    /// Returns a (addresses, address) tuple representing the selected input addresses and the remainder address if needed.
    fn select_inputs<'a>(
        &self,
        threshold: u64,
        account: &'a Account,
        address: &'a Address,
    ) -> crate::Result<(Vec<Address>, Option<&'a Address>)> {
        let mut available_addresses = vec![];
        let available_addresses_iter = account.addresses().iter().filter(|a| a != &address);
        for available_address in available_addresses_iter {
            available_addresses.push(available_address.clone());
        }
        let addresses = input_selection::select_input(threshold, &mut available_addresses)?;
        let remainder = if addresses.iter().fold(0, |acc, a| acc + a.balance()) > threshold {
            account.latest_address()
        } else {
            None
        };
        Ok((addresses, remainder))
    }

    /// Send messages.
    pub async fn transfer(&self, transfer_obj: Transfer) -> crate::Result<Message> {
        // validate the transfer
        if *transfer_obj.amount() == 0 {
            return Err(crate::WalletError::ZeroAmount);
        }

        // prepare the transfer getting some needed objects and values
        let value: u64 = *transfer_obj.amount();
        let account_id: AccountIdentifier = self.account_id.clone().into();
        let mut account = crate::storage::get_account(&self.storage_path, account_id)?;
        let client = get_client(account.client_options());

        // select the input addresses and check if a remainder address is needed
        let (input_addresses, remainder_address) =
            self.select_inputs(*transfer_obj.amount(), &account, transfer_obj.address())?;

        let mut utxos = vec![];
        for utxo_output in &input_addresses {
            let address = utxo_output.address();
            let address_outputs = client.get_address().outputs(&address).await?;
            let mut outputs = vec![];
            for output in address_outputs.iter() {
                let output = client.get_output(output).await?;
                outputs.push(output);
            }
            utxos.extend(outputs.into_iter());
        }

        let mut utxo_inputs: Vec<Input> = vec![];
        let mut utxo_outputs: Vec<Output> = vec![];
        let mut current_output_sum = 0;
        for utxo in utxos {
            utxo_inputs.push(
                UTXOInput::new(
                    TransactionId::new(
                        utxo.transaction_id
                            .as_bytes()
                            .try_into()
                            .map_err(|_| crate::WalletError::InvalidTransactionIdLength)?,
                    ),
                    utxo.output_index,
                )
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
                .into(),
            );
            let utxo_amount = if current_output_sum + utxo.amount > value {
                value - utxo.amount
            } else {
                utxo.amount
            };
            let utxo_address = if current_output_sum == value {
                remainder_address
                    .map(|a| a.address().clone())
                    .expect("remainder address not defined")
            } else {
                utxo.address
            };
            current_output_sum += utxo.amount;
            utxo_outputs.push(
                SignatureLockedSingleOutput::new(
                    utxo_address,
                    NonZeroU64::new(utxo_amount)
                        .ok_or_else(|| anyhow::anyhow!("invalid amount"))?,
                )
                .into(),
            );
        }

        let (parent1, parent2) = client.get_tips().await?;

        let stronghold_account =
            crate::with_stronghold_from_path(&self.storage_path, |stronghold| {
                stronghold.account_get_by_id(account.id())
            })?;

        let mut essence_builder = TransactionEssence::builder();
        for output in utxo_outputs.into_iter() {
            essence_builder = essence_builder.add_output(output);
        }
        for input in utxo_inputs.into_iter() {
            essence_builder = essence_builder.add_input(input);
        }
        let essence = essence_builder
            .finish()
            .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;
        let transaction = Transaction::builder()
            .with_essence(essence)
            .finish()
            .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;

        let message = IotaMessage::builder()
            .with_parent1(parent1)
            .with_parent2(parent2)
            .with_payload(Payload::Transaction(Box::new(transaction)))
            .finish()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let message_id = client.post_message(&message).await?;
        let message = client.get_message().data(&message_id).await?;

        account.append_messages(vec![Message::from_iota_message(message_id, &message)?]);
        crate::storage::with_adapter(&self.storage_path, |storage| {
            storage.set(account_id, serde_json::to_string(&account)?)
        })?;

        Ok(Message::from_iota_message(message_id, &message)?)
    }

    /// Retry messages.
    pub fn retry(&self, message_id: &MessageId) -> crate::Result<Message> {
        let account: Account =
            crate::storage::get_account(&self.storage_path, self.account_id.clone().into())?;
        let message = account
            .get_message(message_id)
            .ok_or_else(|| anyhow::anyhow!("transaction with the given id not found"));
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptionsBuilder;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn account_sync() {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
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
