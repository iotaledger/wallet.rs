// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod balance;

use serde_json::Value;

use self::balance::process_balance;
use crate::{context::Context, error::Error};

pub async fn process_checks<'a>(context: &Context<'a>, checks: &Value) -> Result<(), Error> {
    log::info!("Processing checks.");

    context.account_manager.sync(None).await?;

    if let Some(checks) = checks.as_array() {
        for check in checks {
            if let Some(balance) = check.get("balance") {
                process_balance(context, balance).await?;
            } else {
                return Err(Error::InvalidField("check"));
            }
        }
    } else {
        return Err(Error::InvalidField("checks"));
    }

    Ok(())
}
