// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder,
    },
    payload::milestone::MilestoneIndex,
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        constants::DEFAULT_EXPIRATION_TIME,
        handle::AccountHandle,
        operations::transfer::{
            high_level::minimum_storage_deposit::minimum_storage_deposit_basic_native_tokens, TransferResult,
        },
        TransferOptions,
    },
    Error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// address with amount for `send_micro_transaction()`
pub struct AddressWithMicroAmount {
    /// Bech32 encoded address
    pub address: String,
    /// Amount below the minimum storage deposit
    pub amount: u64,
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    pub return_address: Option<String>,
    /// Expiration in seconds, after which the output will be available for the sender again, if not spent by the
    /// receiver before. Default is 1 day
    pub expiration: Option<u32>,
}

impl AccountHandle {
    /// Function to send micro transactions by using the [StorageDepositReturnUnlockCondition] with an
    /// [ExpirationUnlockCondition]. Will call [AccountHandle.send()](crate::account::handle::AccountHandle.send),
    /// the options can define the RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = vec![AddressWithMicroAmount{
    ///    address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///    amount: 1,
    ///    return_address: None,
    ///    expiration: None,
    /// }];
    ///
    /// let transfer_result = account_handle.send_micro_transaction(outputs, None ).await?;
    ///
    /// println!(
    ///    "Transaction: {} Message sent: http://localhost:14265/api/v2/messages/{}",
    ///    transfer_result.transaction_id,
    ///    transfer_result.message_id.expect("No message created yet")
    /// );
    /// ```
    pub async fn send_micro_transaction(
        &self,
        addresses_with_micro_amount: Vec<AddressWithMicroAmount>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        let account_addresses = self.list_addresses().await?;
        let return_address = account_addresses.first().ok_or(Error::FailedToGetRemainder)?;

        let (local_time, _) = self.get_time_and_milestone_checked().await?;
        let expiration_time = local_time as u32 + DEFAULT_EXPIRATION_TIME;

        let mut outputs = Vec::new();
        for address_with_amount in addresses_with_micro_amount {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_with_amount.address)?;
            // get minimum required amount for such an output, so we don't lock more than required
            // We have to check it for every output individually, because different address types and amount of
            // different native tokens require a differen storage deposit
            let storage_deposit_amount = minimum_storage_deposit_basic_native_tokens(
                &byte_cost_config,
                &address,
                &return_address.address.inner,
                None,
            )?;

            outputs.push(
                // Add address_and_amount.amount+storage_deposit_amount, so receiver can get address_and_amount.amount
                BasicOutputBuilder::new_with_amount(address_with_amount.amount + storage_deposit_amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                    .add_unlock_condition(UnlockCondition::StorageDepositReturn(
                        // We send the storage_deposit_amount back to the sender, so only the additional amount is sent
                        StorageDepositReturnUnlockCondition::new(return_address.address.inner, storage_deposit_amount)?,
                    ))
                    .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                        address,
                        // 0 means it's ignored during validation
                        MilestoneIndex::new(0),
                        expiration_time,
                    )?))
                    .finish_output()?,
            )
        }
        self.send(outputs, options).await
    }
}
