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

/// Function to send native tokens in basic outputs with a [StorageDepositReturnUnlockCondition] and
/// [ExpirationUnlockCondition], so the storage deposit gets back to the sender and also that the sender gets access to
/// the output again after a defined time (default 1 day),
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
/// let res = account_handle
///     .send_native_tokens(
///         outputs,
///         Some(TransferOptions {
///             remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
///             ..Default::default()
///         }),
///     )
///     .await?;
/// println!("Transaction created: {}", res.1);
/// if let Some(message_id) = res.0 {
///     println!("Message sent: {}", message_id);
/// }
/// ```
pub async fn send_native_tokens(
    account_handle: &AccountHandle,
    addresses_native_tokens: Vec<AddressNativeTokens>,
    options: Option<TransferOptions>,
) -> crate::Result<TransferResult> {
    let account_addresses = account_handle.list_addresses().await?;
    let return_address = account_addresses.first().ok_or(Error::FailedToGetRemainder)?;

    // todo: get minimum required amount for such an output, so we don't lock more than required
    let storage_deposit_amount = 1_500_000;

    // todo: how to prevent locked funds if the system time is wrong? Compare time with the milestone
    // or only use milestones? `0` value of the `Milestone Index` field is a
    // special flag that signals to the validation that this check   must be
    // ignored.
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32
        + 86400;

    let mut outputs = Vec::new();
    for address_and_amount in addresses_native_tokens {
        let address = Address::try_from_bech32(&address_and_amount.address)?;
        outputs.push(Output::Basic(
            BasicOutputBuilder::new(storage_deposit_amount)?
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                .add_unlock_condition(UnlockCondition::StorageDepositReturn(
                    StorageDepositReturnUnlockCondition::new(return_address.address.inner, storage_deposit_amount)?,
                ))
                .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    address,
                    MilestoneIndex::new(0),
                    expiration_time,
                )?))
                .finish()?,
        ))
    }
    account_handle.send(outputs, options).await
}
