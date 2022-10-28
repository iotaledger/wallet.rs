// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::{context::Context, error::Error};

pub async fn process_balance<'a>(context: &Context<'a>, balance: &Value) -> Result<(), Error> {
    let account_index = if let Some(account_index) = balance.get("account") {
        if let Some(account_index) = account_index.as_u64() {
            account_index as usize
        } else {
            return Err(Error::InvalidField("account_index"));
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

    if let Some(account) = context.account_manager.get_accounts().await?.get(account_index) {
        let balance = account.balance().await?;

        if balance.base_coin.available != amount {
            return Err(Error::Check(format!(
                "incorrect balance for account {}: expected {}, got {}",
                account_index, amount, balance.base_coin.available
            )));
        }
    } else {
        return Err(Error::InvalidField("account"));
    };

    Ok(())
}
