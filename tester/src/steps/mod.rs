// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod transaction;

use serde_json::Value;

use self::transaction::process_transaction;
use crate::{context::Context, error::Error};

pub async fn process_steps(context: &Context, steps: &Value) -> Result<(), Error> {
    // if let Some(transaction) = json.get("transaction") {
    //     process_transaction(context, transaction).await?;
    // }

    if let Some(steps) = steps.as_array() {
        for step in steps {
            if let Some(transaction) = step.get("transaction") {
                process_transaction(context, transaction).await?;
            } else {
                return Err(Error::InvalidField("step"));
            }
        }
    } else {
        return Err(Error::InvalidField("steps"));
    }

    Ok(())
}
