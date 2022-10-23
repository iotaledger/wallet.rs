// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_wallet::AddressWithAmount;
use serde_json::Value;
use tokio::time;

use crate::{context::Context, error::Error};

pub async fn process_accounts<'a>(context: &Context<'a>, accounts: &Value) -> Result<(), Error> {
    if let Some(accounts) = accounts.as_array() {
        for account in accounts {
            let amount = if let Some(amount) = account.as_u64() {
                amount
            } else {
                return Err(Error::InvalidField("account"));
            };

            let account = context.account_manager.create_account().finish().await?;

            if amount != 0 {
                context
                    .faucet_account
                    .send_amount(
                        vec![AddressWithAmount {
                            address: account.addresses().await?[0].address().to_bech32(),
                            amount: amount,
                        }],
                        None,
                    )
                    .await?;
            }
        }

        time::sleep(Duration::from_secs(10)).await;
        context.faucet_account.sync(None).await?;
    }

    Ok(())
}
