// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod accounts;

use serde_json::Value;

use self::accounts::process_accounts;
use crate::{context::Context, error::Error};

pub async fn process_fixtures<'a>(context: &Context<'a>, fixtures: &Value) -> Result<(), Error> {
    log::info!("Processing fixtures.");

    if let Some(accounts) = fixtures.get("accounts") {
        process_accounts(context, accounts).await?;
    } else {
        return Err(Error::InvalidField("fixtures"));
    }

    Ok(())
}
