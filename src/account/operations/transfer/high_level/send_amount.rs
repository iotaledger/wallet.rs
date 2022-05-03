// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_message::{
    address::Address,
    output::{
        unlock_condition::{AddressUnlockCondition, UnlockCondition},
        BasicOutputBuilder,
    },
};
use serde::{Deserialize, Serialize};

use crate::account::{handle::AccountHandle, operations::transfer::TransferResult, TransferOptions};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// address with amount for `send_amount()`
pub struct AddressWithAmount {
    /// Bech32 encoded address
    pub address: String,
    /// Amount
    pub amount: u64,
}

impl AccountHandle {
    /// Function to create basic outputs with which we then will call
    /// [AccountHandle.send()](crate::account::handle::AccountHandle.send), the options can define the
    /// RemainderValueStrategy or custom inputs.
    /// Address needs to be Bech32 encoded
    /// ```ignore
    /// let outputs = vec![AddressWithAmount{
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     amount: 1_000_000,
    /// }];
    ///
    /// let res = account_handle.send_amount(outputs, None ).await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn send_amount(
        &self,
        addresses_with_amount: Vec<AddressWithAmount>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        let mut outputs = Vec::new();
        for address_with_amount in addresses_with_amount {
            outputs.push(
                BasicOutputBuilder::new_with_amount(address_with_amount.amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        Address::try_from_bech32(&address_with_amount.address)?.1,
                    )))
                    .finish_output()?,
            )
        }
        self.send(outputs, options).await
    }
}
