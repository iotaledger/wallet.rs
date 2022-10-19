// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    block::{
        address::Address,
        output::{
            unlock_condition::{
                AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
            },
            BasicOutputBuilder, NativeToken, TokenId,
        },
    },
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        constants::DEFAULT_EXPIRATION_TIME,
        handle::AccountHandle,
        operations::transaction::{
            high_level::minimum_storage_deposit::minimum_storage_deposit_basic_native_tokens, Transaction,
        },
        TransactionOptions,
    },
    Error, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Address, amount and native tokens for `send_native_tokens()`
pub struct AddressNativeTokens {
    /// Bech32 encoded address
    pub address: String,
    /// Native tokens
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<(TokenId, U256)>,
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    #[serde(rename = "returnAddress")]
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
    ///     address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
    ///     native_tokens: vec![(
    ///         TokenId::from_str("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?,
    ///         U256::from(50),
    ///     )],
    ///     ..Default::default()
    /// }];
    ///
    /// let tx = account_handle.send_native_tokens(outputs, None).await?;
    /// println!("Transaction created: {}", tx.transaction_id);
    /// if let Some(block_id) = tx.block_id {
    ///     println!("Block sent: {}", block_id);
    /// }
    /// ```
    pub async fn send_native_tokens(
        &self,
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        let prepared_transaction = self
            .prepare_send_native_tokens(addresses_native_tokens, options)
            .await?;
        self.sign_and_submit_transaction(prepared_transaction).await
    }

    /// Function to prepare the transaction for
    /// [AccountHandle.send_native_tokens()](crate::account::handle::AccountHandle.send_native_tokens)
    async fn prepare_send_native_tokens(
        &self,
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_send_native_tokens");
        let rent_structure = self.client.get_rent_structure()?;
        let token_supply = self.client.get_token_supply()?;

        let account_addresses = self.addresses().await?;
        let return_address = account_addresses.first().ok_or(Error::FailedToGetRemainder)?;

        let local_time = self.client.get_time_checked()?;

        let mut outputs = Vec::new();
        for address_with_amount in addresses_native_tokens {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_with_amount.address)?;
            // get minimum required amount for such an output, so we don't lock more than required
            // We have to check it for every output individually, because different address types and amount of
            // different native tokens require a different storage deposit
            let storage_deposit_amount = minimum_storage_deposit_basic_native_tokens(
                &rent_structure,
                &address,
                &return_address.address.inner,
                Some(address_with_amount.native_tokens.clone()),
                token_supply,
            )?;

            let expiration_time = match address_with_amount.expiration {
                Some(expiration_time) => local_time + expiration_time,
                None => local_time + DEFAULT_EXPIRATION_TIME,
            };

            outputs.push(
                BasicOutputBuilder::new_with_amount(storage_deposit_amount)?
                    .with_native_tokens(
                        address_with_amount
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
                        StorageDepositReturnUnlockCondition::new(
                            return_address.address.inner,
                            storage_deposit_amount,
                            token_supply,
                        )?,
                    ))
                    .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                        return_address.address.inner,
                        expiration_time,
                    )?))
                    .finish_output(token_supply)?,
            )
        }

        self.prepare_transaction(outputs, options).await
    }
}
