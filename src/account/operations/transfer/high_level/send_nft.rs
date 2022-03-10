// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        NftId, NftOutputBuilder, Output,
    },
};
// use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address and nft for `send_nft()`
pub struct AddressAndNftId {
    /// Bech32 encoded address
    pub address: String,
    /// Nft id
    #[serde(rename = "nftId")]
    pub nft_id: NftId,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a [StorageDepositReturnUnlockCondition] and
    /// [ExpirationUnlockCondition], so the storage deposit gets back to the sender and also that the sender gets access
    /// to the output again after a defined time (default 1 day),
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let nft_id: [u8; 38] =
    ///     hex::decode("08e68f7616cd4948efebc6a77c4f93aed770ac53860100000000000000000000000000000000")?
    ///         .try_into()
    ///         .unwrap();
    /// let outputs = vec![AddressAndNftId {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     nft_id: NftId::new(nft_id),
    /// }];
    ///
    /// let res = account_handle.send_nft(outputs, None).await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn send_nft(
        &self,
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let mut outputs = Vec::new();
        for address_and_nft_id in addresses_nft_ids {
            let (_bech32_hrp, address) = Address::try_from_bech32(&address_and_nft_id.address)?;
            // todo get nft output from account, build new output with same fields and native tokens, just updated
            // address unlock conditions
            outputs.push(Output::Nft(
                NftOutputBuilder::new(3_000_000, NftId::from([0; 20]))?
                    // NftOutputBuilder::new(input_nft.amount(), input_nft.nft_id())?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                    .finish()?,
            ));
        }
        self.send(outputs, options).await
    }
}
