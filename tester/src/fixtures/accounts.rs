// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_wallet::iota_client::request_funds_from_faucet;
use serde_json::Value;
use tokio::time;

use crate::{context::Context, error::Error};

pub async fn process_accounts(context: &Context, accounts: &Value) -> Result<(), Error> {
    if let Some(accounts) = accounts.as_array() {
        // TODO improve by doing one summed request and dispatching
        for account in accounts {
            let amount = if let Some(amount) = account.as_u64() {
                amount
            } else {
                return Err(Error::InvalidField("account"));
            };

            let account = context.account_manager.create_account().finish().await?;

            if amount != 0 {
                let _res = request_funds_from_faucet(
                    "https://faucet.testnet.shimmer.network/api/enqueue",
                    &account.addresses().await?[0].address().to_bech32(),
                )
                .await?;
            }
        }

        time::sleep(Duration::from_secs(10)).await;
    }

    Ok(())
}
