// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::{context::Context, error::Error};

pub async fn process_checks(context: &Context, checks: &Value) -> Result<(), Error> {
    context.account_manager.sync(None).await?;
    println!("{}", checks);

    if let Some(checks) = checks.as_array() {
        for check in checks {
            if let Some(balance) = check.get("balance") {
                let account = if let Some(account) = balance.get("account") {
                    if let Some(account) = account.as_u64() {
                        account as usize
                    } else {
                        return Err(Error::InvalidField("account"));
                    }
                } else {
                    return Err(Error::MissingField("account"));
                };

                let amount = if let Some(amount) = balance.get("amount") {
                    if let Some(amount) = amount.as_u64() {
                        amount
                    } else {
                        return Err(Error::InvalidField("amount"));
                    }
                } else {
                    return Err(Error::MissingField("amount"));
                };

                println!("{}", account);
                println!("{}", amount);

                if let Some(account) = context.account_manager.get_accounts().await?.get(account) {
                    let balance = account.balance().await?;

                    if balance.base_coin.available != amount {
                        println!("TEST FAILURE");
                    }
                } else {
                    return Err(Error::InvalidField("account"));
                };
            } else {
                return Err(Error::InvalidField("test"));
            }
        }
    }

    Ok(())
}
