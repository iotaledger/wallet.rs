// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_wallet::iota_client::request_funds_from_faucet;

use serde_json::Value;
use tokio::time;

use crate::{context::Context, error::Error};

pub async fn process_fixtures(context: &Context, fixtures: &Value) -> Result<(), Error> {
    println!("{}", fixtures);

    let res = request_funds_from_faucet(
        "https://faucet.testnet.shimmer.network/api/enqueue",
        &context.account_manager.get_accounts().await?[0].addresses().await?[0]
            .address()
            .to_bech32(),
    )
    .await?;

    println!("{:?}", res);

    if let Some(accounts) = fixtures.get("accounts") {
        println!("{}", accounts);
        if let Some(accounts) = accounts.as_array() {
            let mut amounts = Vec::new();

            for account in accounts {
                if let Some(amount) = account.as_u64() {
                    amounts.push(amount);
                } else {
                    return Err(Error::InvalidField("account"));
                }
            }

            // TODO improve by doing one summed request and dispatching
            for amount in amounts {
                let account = context.account_manager.create_account().finish().await?;

                if amount != 0 {
                    let res = request_funds_from_faucet(
                        "https://faucet.testnet.shimmer.network/api/enqueue",
                        &account.addresses().await?[0].address().to_bech32(),
                    )
                    .await?;

                    println!("{:?}", res);
                }
            }

            time::sleep(Duration::from_secs(10)).await;
        }
    } else {
        return Err(Error::InvalidField("accounts"));
    }

    Ok(())
}
