// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use iota_wallet::iota_client::block::output::{
    unlock_condition::{AddressUnlockCondition, UnlockCondition},
    BasicOutputBuilder,
};
use serde_json::Value;
use tokio::time;

use crate::{context::Context, error::Error};

pub async fn process_transaction(context: &Context, transaction: &Value) -> Result<(), Error> {
    context.account_manager.sync(None).await?;

    if let Some(inputs) = transaction.get("inputs") {
        if let Some(inputs) = inputs.as_array() {
            for _input in inputs {}
        } else {
            return Err(Error::InvalidField("inputs"));
        }
    }

    let mut outputs = Vec::new();

    if let Some(json_outputs) = transaction.get("outputs") {
        if let Some(json_outputs) = json_outputs.as_array() {
            for output in json_outputs {
                if let Some(_dto) = output.get("dto") {
                } else if let Some(simple) = output.get("simple") {
                    let account = if let Some(account) = simple.get("account") {
                        if let Some(account) = account.as_u64() {
                            account as usize
                        } else {
                            return Err(Error::InvalidField("account"));
                        }
                    } else {
                        return Err(Error::MissingField("account"));
                    };

                    let amount = if let Some(amount) = simple.get("amount") {
                        if let Some(amount) = amount.as_u64() {
                            amount
                        } else {
                            return Err(Error::InvalidField("amount"));
                        }
                    } else {
                        return Err(Error::MissingField("amount"));
                    };

                    let address = if let Some(account) = context.account_manager.get_accounts().await?.get(account) {
                        account.addresses().await?[0].address().as_ref().clone()
                    } else {
                        return Err(Error::InvalidField("account"));
                    };

                    // TODO unwrap
                    let simple_output = BasicOutputBuilder::new_with_amount(amount)
                        .unwrap()
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                        .finish_output(context.protocol_parameters.token_supply())
                        .unwrap();

                    outputs.push(simple_output);
                } else {
                    return Err(Error::InvalidField("output"));
                }
            }
        } else {
            return Err(Error::InvalidField("outputs"));
        }
    } else {
        return Err(Error::MissingField("outputs"));
    }

    let _transaction = context.account_manager.get_accounts().await?[0]
        .send(outputs, None)
        .await?;

    time::sleep(Duration::from_secs(10)).await;

    Ok(())
}
