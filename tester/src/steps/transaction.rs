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

pub async fn process_transaction<'a>(context: &Context<'a>, transaction: &Value) -> Result<(), Error> {
    let account_from_index = if let Some(from) = transaction.get("from") {
        if let Some(from) = from.as_u64() {
            from as usize
        } else {
            return Err(Error::InvalidField("from"));
        }
    } else {
        return Err(Error::MissingField("from"));
    };

    let accounts = context.account_manager.get_accounts().await?;

    let account_from = if let Some(account_from) = accounts.get(account_from_index) {
        account_from
    } else {
        return Err(Error::InvalidField("from"));
    };

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
                        *account.addresses().await?[0].address().as_ref()
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

    match account_from.send(outputs, None).await {
        Ok(sent_transaction) => {
            if let Some(confirmation) = transaction.get("confirmation") {
                if let Some(confirmation) = confirmation.as_bool() {
                    if confirmation {
                        if let Some(block_id) = sent_transaction.block_id {
                            account_from.retry_until_included(&block_id, Some(1), None).await?;
                        } else {
                            time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                } else {
                    return Err(Error::InvalidField("confirmation"));
                }
            }

            account_from.sync(None).await?;
        }
        Err(e) => {
            if let Some(error) = transaction.get("error") {
                if let Some(error) = error.as_str() {
                    if !e.to_string().contains(error) {
                        return Err(Error::Unexpected {
                            expected: error.into(),
                            actual: e.to_string(),
                        });
                    }
                } else {
                    return Err(Error::InvalidField("error"));
                }
            } else {
                return Err(e)?;
            }
        }
    }

    Ok(())
}
