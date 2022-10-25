// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::AddressWithAmount;
use serde_json::Value;

use crate::{context::Context, error::Error};

pub async fn process_account<'a>(context: &Context<'a>, account: &Value) -> Result<(), Error> {
    let amount = if let Some(amount) = account.as_u64() {
        amount
    } else {
        return Err(Error::InvalidField("account"));
    };

    let account = context.account_manager.create_account().finish().await?;

    if amount != 0 {
        let transaction = context
            .faucet_account
            .send_amount(
                vec![AddressWithAmount {
                    address: account.addresses().await?[0].address().to_bech32(),
                    amount: amount,
                }],
                None,
            )
            .await?;

        if let Some(block_id) = transaction.block_id {
            context
                .faucet_account
                .retry_until_included(&block_id, Some(1), None)
                .await?;
        }
    }

    Ok(())
}
