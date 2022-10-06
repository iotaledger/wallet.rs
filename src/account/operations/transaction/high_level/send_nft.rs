// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::PreparedTransactionData,
    block::{
        address::Address,
        output::{
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            NftId, NftOutputBuilder, Output,
        },
    },
};
// use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::account::{handle::AccountHandle, operations::transaction::Transaction, TransactionOptions};

/// Address and nft for `send_nft()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressAndNftId {
    /// Bech32 encoded address
    pub address: String,
    /// Nft id
    #[serde(rename = "nftId")]
    pub nft_id: NftId,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a
    /// [`StorageDepositReturnUnlockCondition`](iota_client::block::output::unlock_condition::
    /// StorageDepositReturnUnlockCondition) and [`ExpirationUnlockCondition`](iota_client::block::output::
    /// unlock_condition::ExpirationUnlockCondition), so the storage deposit gets back to the sender and also that
    /// the sender gets access to the output again after a defined time (default 1 day), Calls
    /// [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy. Custom inputs will be replaced with the required nft inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = vec![AddressAndNftId {
    ///     address: "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu".to_string(),
    ///     nft_id: NftId::from_str("04f9b54d488d2e83a6c90db08ae4b39651bbba8a")?,
    /// }];
    ///
    /// let transaction = account.send_nft(outputs, None).await?;
    ///
    /// println!(
    /// "Transaction: {} Block sent: http://localhost:14265/api/core/v2/blocks/{}",
    /// transaction.transaction_id,
    /// transaction.block_id.expect("no block created yet")
    /// );
    /// ```
    pub async fn send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<Transaction> {
        let prepared_transaction = self.prepare_send_nft(addresses_nft_ids, options).await?;
        self.sign_and_submit_transaction(prepared_transaction).await
    }

    /// Function to prepare the transaction for
    /// [AccountHandle.send_nft()](crate::account::handle::AccountHandle.send_nft)
    async fn prepare_send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_send_nft");

        let unspent_outputs = self.unspent_outputs(None).await?;
        let token_supply = self.client.get_token_supply()?;

        let mut outputs = Vec::new();

        for address_and_nft_id in addresses_nft_ids {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_and_nft_id.address)?;
            // Find nft output from the inputs
            if let Some(nft_output_data) = unspent_outputs.iter().find(|o| {
                if let Output::Nft(nft_output) = &o.output {
                    address_and_nft_id.nft_id == nft_output.nft_id().or_from_output_id(o.output_id)
                } else {
                    false
                }
            }) {
                if let Output::Nft(nft_output) = &nft_output_data.output {
                    // Set the nft id and new address unlock condition
                    let nft_builder = NftOutputBuilder::from(nft_output)
                        .with_nft_id(address_and_nft_id.nft_id)
                        .with_unlock_conditions(vec![UnlockCondition::Address(AddressUnlockCondition::new(address))]);
                    outputs.push(nft_builder.finish_output(token_supply)?);
                }
            } else {
                return Err(crate::Error::NftNotFoundInUnspentOutputs);
            };
        }

        self.prepare_transaction(outputs, options).await
    }
}
