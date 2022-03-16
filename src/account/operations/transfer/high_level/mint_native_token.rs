// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions},
    Error,
};

use iota_client::bee_message::address::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Address and nft for `mint_native_token()`
pub struct NativeTokenOptions {
    /// Bech32 encoded address. Needs to be an account address. Default will use the
    /// first address of the account
    #[serde(rename = "accountAddress")]
    pub account_address: Option<String>,
    /// Token tag
    #[serde(rename = "tokenTag")]
    pub token_tag: String,
    /// Circulating supply
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: U256,
    /// Maximum supply
    #[serde(rename = "maxiumSupply")]
    pub maxium_supply: U256,
}

impl AccountHandle {
    /// Function to send native tokens in basic outputs with a [StorageDepositReturnUnlockCondition] and
    /// [ExpirationUnlockCondition], so the storage deposit gets back to the sender and also that the sender gets access
    /// to the output again after a defined time (default 1 day),
    /// Calls [AccountHandle.send()](crate::account::handle::AccountHandle.send) internally, the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = NativeTokenOptions {
    ///     account_address: Some("atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string()),
    ///     token_tag: "some_token_tag".to_string(),
    ///     circulating_supply: U256::from(100),
    ///     maxium_supply: U256::from(100),
    /// };
    ///
    /// let res = account_handle.mint_native_token(outputs, None,).await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn mint_native_token(
        &self,
        native_token_options: NativeTokenOptions,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let account_addresses = self.list_addresses().await?;
        // the address needs to be from the account, because for the minting we need to sign transactions from it
        let controller_address = match native_token_options.account_address {
            Some(address) => {
                let (_bech32_hrp, address) = Address::try_from_bech32(&address)?;
                if account_addresses
                    .binary_search_by_key(&address, |address| address.address.inner)
                    .is_err()
                {
                    return Err(Error::AddressNotFoundInAccount);
                }
                address
            }
            None => {
                account_addresses
                    .first()
                    // todo other error message
                    .ok_or(Error::FailedToGetRemainder)?
                    .address
                    .inner
            }
        };
        // todo check if an alias output already exists
        // otherwise create an alias output first
        // create foundry output with minted native tokens
        // mint native tokens to basic output of the provided address?

        // let mut outputs = Vec::new();
        //     outputs.push(nft_output);
        // self.send(outputs, options).await
        todo!()
    }
}
