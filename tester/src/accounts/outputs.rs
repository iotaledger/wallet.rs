// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::account::AccountHandle;
use serde_json::Value;

use crate::{accounts::amount::process_amount, context::Context, error::Error};

pub async fn process_outputs<'a>(
    context: &Context<'a>,
    account: &AccountHandle,
    outputs: &[Value],
) -> Result<(), Error> {
    for output in outputs {
        if output.is_u64() {
            process_amount(context, account, output.as_u64().unwrap()).await?;
        }
    }

    Ok(())
}
