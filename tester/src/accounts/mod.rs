// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod account;
mod amount;
mod outputs;

use serde_json::Value;

use self::account::process_account;
use crate::{context::Context, error::Error};

pub async fn process_accounts<'a>(context: &Context<'a>, accounts: &Value) -> Result<(), Error> {
    log::info!("Processing accounts.");

    if let Some(accounts) = accounts.as_array() {
        for account in accounts {
            process_account(context, account).await?;
        }
    } else {
        return Err(Error::InvalidField("accounts"));
    }

    Ok(())
}
