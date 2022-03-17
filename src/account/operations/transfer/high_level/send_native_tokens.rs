// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        constants::DEFAULT_EXPIRATION_TIME,
        handle::AccountHandle,
        operations::transfer::{
            high_level::minimum_storage_deposit::minimum_storage_deposit_basic_native_tokens, TransferResult,
        },
        TransferOptions,
    },
    Error, Result,
};

use iota_client::bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder, ByteCostConfigBuilder, NativeToken, Output, TokenId,
    },
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Address, amount and native tokens for `send_native_tokens()`
pub struct AddressNativeTokens {
    /// Bech32 encoded address
    pub address: String,
    /// Native tokens
    pub native_tokens: Vec<(TokenId, U256)>,
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
    /// let outputs = vec![AddressNativeTokens {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     native_tokens: vec![(
    ///         TokenId::from_str("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?,
    ///         U256::from(50),
    ///     )],
    ///     ..Default::default()
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
        let rent_structure = self.client.get_rent_structure().await?;
        let byte_cost_config = ByteCostConfigBuilder::new()
            .byte_cost(rent_structure.v_byte_cost)
            .key_factor(rent_structure.v_byte_factor_key)
            .data_factor(rent_structure.v_byte_factor_data)
            .finish();

        let account_addresses = self.list_addresses().await?;
        let return_address = account_addresses.first().ok_or(Error::FailedToGetRemainder)?;

        let (local_time, _) = self.get_time_and_milestone_checked().await?;
        let expiration_time = local_time as u32 + DEFAULT_EXPIRATION_TIME;

        let mut outputs = Vec::new();
        for address_and_amount in addresses_native_tokens {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_and_amount.address)?;
            // get minimum required amount for such an output, so we don't lock more than required
            // We have to check it for every output individually, because different address types and amount of
            // different native tokens require a differen storage deposit
            let storage_deposit_amount = minimum_storage_deposit_basic_native_tokens(
                &byte_cost_config,
                &address,
                &return_address.address.inner,
                &address_and_amount.native_tokens,
            )?;

            outputs.push(Output::Basic(
                BasicOutputBuilder::new(storage_deposit_amount)?
                    .with_native_tokens(
                        address_and_amount
                            .native_tokens
                            .into_iter()
                            .map(|(id, amount)| {
                                NativeToken::new(id, amount).map_err(|e| crate::Error::ClientError(Box::new(e.into())))
                            })
                            .collect::<Result<Vec<NativeToken>>>()?,
                    )
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
