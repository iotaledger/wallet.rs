// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions},
    Error,
};

use iota_client::bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder, Output, TokenId,
    },
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

const FIVE_MINUTES_IN_SECONDS: u64 = 300;
// One day in seconds
const DEFAULT_EXPIRATION_TIME: u32 = 86400;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address, amount and native tokens for `send_native_tokens()`
pub struct AddressNativeTokens {
    /// Bech32 encoded address
    pub address: String,
    /// Native tokens
    pub native_tokens: HashMap<TokenId, U256>,
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    pub return_address: Option<String>,
    /// Expiration in seconds, after which the output will be available for the sender again, if not spent by the
    /// receiver before. Default is 1 day
    pub expiration: Option<u32>,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a [StorageDepositReturnUnlockCondition] and
    /// [ExpirationUnlockCondition], so the storage deposit gets back to the sender and also that the sender gets access
    /// to the output again after a defined time (default 1 day),
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let token_id: [u8; 38] =
    ///     hex::decode("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?
    ///         .try_into()
    ///         .unwrap();
    /// let mut native_tokens = HashMap::new();
    /// native_tokens.insert(TokenId::new(token_id), U256::from(50));
    /// let outputs = vec![AddressAmountNativeTokens {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     native_tokens,
    /// }];
    ///
    /// let res = account_handle.send_native_tokens(outputs, None).await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn send_native_tokens(
        &self,
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let account_addresses = self.list_addresses().await?;
        let return_address = account_addresses.first().ok_or(Error::FailedToGetRemainder)?;

        // todo: get minimum required amount for such an output, so we don't lock more than required
        let storage_deposit_amount = 1_500_000;

        let local_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let latest_ms_timestamp = self.client.get_info().await?.nodeinfo.status.latest_milestone_timestamp;

        // Check the local time is in the range of +-5 minutes of the node to prevent locking funds by accident
        if !(latest_ms_timestamp - FIVE_MINUTES_IN_SECONDS..latest_ms_timestamp + FIVE_MINUTES_IN_SECONDS)
            .contains(&local_time)
        {
            return Err(Error::TimeNotSynced(local_time, latest_ms_timestamp));
        }
        let expiration_time = local_time as u32 + DEFAULT_EXPIRATION_TIME;

        let mut outputs = Vec::new();
        for address_and_amount in addresses_native_tokens {
            let address = Address::try_from_bech32(&address_and_amount.address)?;
            outputs.push(Output::Basic(
                BasicOutputBuilder::new(storage_deposit_amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                    .add_unlock_condition(UnlockCondition::StorageDepositReturn(
                        // We send the full storage_deposit_amount back to the sender, so only the native tokens are
                        // sent
                        StorageDepositReturnUnlockCondition::new(return_address.address.inner, storage_deposit_amount)?,
                    ))
                    .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                        address,
                        // 0 means it's ignored during validation
                        MilestoneIndex::new(0),
                        expiration_time,
                    )?))
                    .finish()?,
            ))
        }
        self.send(outputs, options).await
    }
}
