// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::{
    accounts::{amount::process_amount, outputs::process_outputs},
    context::Context,
    error::Error,
};

pub async fn process_account<'a>(context: &Context<'a>, account: &Value) -> Result<(), Error> {
    let created_account = context.account_manager.create_account().finish().await?;

    if account.is_u64() {
        process_amount(context, &created_account, account.as_u64().unwrap()).await?;
    } else if account.is_array() {
        process_outputs(context, &created_account, account.as_array().unwrap()).await?;
    }

    Ok(())
}
