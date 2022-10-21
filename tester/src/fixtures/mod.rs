// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod accounts;

use iota_wallet::iota_client::request_funds_from_faucet;

use serde_json::Value;

use self::accounts::process_accounts;
use crate::{context::Context, error::Error};

pub async fn process_fixtures(context: &Context, fixtures: &Value) -> Result<(), Error> {
    let _res = request_funds_from_faucet(
        "https://faucet.testnet.shimmer.network/api/enqueue",
        &context.account_manager.get_accounts().await?[0].addresses().await?[0]
            .address()
            .to_bech32(),
    )
    .await?;

    if let Some(accounts) = fixtures.get("accounts") {
        process_accounts(context, accounts).await?;
    } else {
        return Err(Error::InvalidField("accounts"));
    }

    Ok(())
}
