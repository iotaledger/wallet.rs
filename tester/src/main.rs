// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use std::time::Duration;

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::{
        block::{
            output::{
                unlock_condition::{AddressUnlockCondition, UnlockCondition},
                BasicOutputBuilder,
            },
            protocol::ProtocolParameters,
        },
        constants::SHIMMER_COIN_TYPE,
        generate_mnemonic, request_funds_from_faucet,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::{fs, time};

use self::error::Error;

struct Context {
    account_manager: AccountManager,
    protocol_parameters: ProtocolParameters,
}

async fn process_fixtures(context: &Context, fixtures: &Value) -> Result<(), Error> {
    println!("{}", fixtures);

    let res = request_funds_from_faucet(
        "https://faucet.testnet.shimmer.network/api/enqueue",
        &context.account_manager.get_accounts().await?[0].addresses().await?[0]
            .address()
            .to_bech32(),
    )
    .await?;

    println!("{:?}", res);

    if let Some(accounts) = fixtures.get("accounts") {
        println!("{}", accounts);
        if let Some(accounts) = accounts.as_array() {
            let mut amounts = Vec::new();

            for account in accounts {
                if let Some(amount) = account.as_u64() {
                    amounts.push(amount);
                } else {
                    return Err(Error::InvalidField("account"));
                }
            }

            // TODO improve by doing one summed request and dispatching
            for amount in amounts {
                let account = context.account_manager.create_account().finish().await?;

                if amount != 0 {
                    let res = request_funds_from_faucet(
                        "https://faucet.testnet.shimmer.network/api/enqueue",
                        &account.addresses().await?[0].address().to_bech32(),
                    )
                    .await?;

                    println!("{:?}", res);
                }
            }

            time::sleep(Duration::from_secs(10)).await;
        }
    } else {
        return Err(Error::InvalidField("accounts"));
    }

    Ok(())
}

async fn process_transactions(context: &Context, transactions: &Value) -> Result<(), Error> {
    println!("{}", transactions);

    if let Some(transactions) = transactions.as_array() {
        for transaction in transactions {
            context.account_manager.sync(None).await?;

            if let Some(inputs) = transaction.get("inputs") {
                if let Some(inputs) = inputs.as_array() {
                    for input in inputs {
                        println!("{}", input);
                    }
                } else {
                    return Err(Error::InvalidField("inputs"));
                }
            }

            let mut outputs = Vec::new();

            if let Some(json_outputs) = transaction.get("outputs") {
                if let Some(json_outputs) = json_outputs.as_array() {
                    for output in json_outputs {
                        if let Some(dto) = output.get("dto") {
                            println!("{}", dto);
                        } else if let Some(simple) = output.get("simple") {
                            println!("{}", simple);

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

                            println!("{}", account);
                            println!("{}", amount);

                            let address =
                                if let Some(account) = context.account_manager.get_accounts().await?.get(account) {
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

            let transaction = context.account_manager.get_accounts().await?[0]
                .send(outputs, None)
                .await?;

            println!("{:?}", transaction);

            time::sleep(Duration::from_secs(10)).await;
        }
    } else {
        return Err(Error::InvalidField("transactions"));
    }

    Ok(())
}

async fn process_tests(context: &Context, tests: &Value) -> Result<(), Error> {
    context.account_manager.sync(None).await?;
    println!("{}", tests);

    if let Some(tests) = tests.as_array() {
        for test in tests {
            if let Some(balance) = test.get("balance") {
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

async fn process_json(context: &Context, json: Value) -> Result<(), Error> {
    if let Some(fixtures) = json.get("fixtures") {
        process_fixtures(context, fixtures).await?;
    }

    if let Some(transactions) = json.get("transactions") {
        process_transactions(context, transactions).await?;
    }

    if let Some(tests) = json.get("tests") {
        process_tests(context, tests).await?;
    }

    Ok(())
}

async fn account_manager() -> Result<AccountManager, Error> {
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(&generate_mnemonic()?)?;

    let client_options = ClientOptions::new()
        .with_node("https://api.testnet.shimmer.network")?
        .with_node_sync_disabled();

    let account_manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    Ok(account_manager)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let account_manager = account_manager().await?;
    account_manager.create_account().finish().await?;
    let protocol_parameters = account_manager.get_accounts().await?[0]
        .client()
        .get_protocol_parameters()?;
    let context = Context {
        account_manager: account_manager,
        protocol_parameters,
    };

    let mut dir = fs::read_dir("json").await?;

    for entry in dir.next_entry().await? {
        let content = fs::read_to_string(entry.path()).await?;
        let json: Value = serde_json::from_str(&content)?;

        println!("{:?}", entry.file_name());
        println!("{}", json);
        process_json(&context, json).await?;
    }

    Ok(())
}
