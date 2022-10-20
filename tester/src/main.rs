// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use std::time::Duration;

use iota_wallet::{
    account::AccountHandle,
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
        request_funds_from_faucet,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::{fs, time};

use self::error::Error;

struct Context {
    _account_manager: AccountManager,
    account: AccountHandle,
    protocol_parameters: ProtocolParameters,
}

async fn process_fixtures(context: &Context, fixtures: &Value) -> Result<(), Error> {
    println!("{}", fixtures);

    if let Some(addresses) = fixtures.get("addresses") {
        println!("{}", addresses);
        if let Some(addresses) = addresses.as_array() {
            let mut amounts = Vec::new();

            for address in addresses {
                if let Some(amount) = address.as_u64() {
                    amounts.push(amount);
                } else {
                    return Err(Error::InvalidField("address"));
                }
            }

            let addresses = context.account.generate_addresses(amounts.len() as u32, None).await?;

            println!("{:?}", addresses);

            // TODO improve by doing one summed request and dispatching
            for (address, _amount) in addresses.iter().zip(amounts.iter()) {
                let res = request_funds_from_faucet(
                    "https://faucet.testnet.shimmer.network/api/enqueue",
                    &address.address().to_bech32(),
                )
                .await?;

                println!("{:?}", res);
            }

            time::sleep(Duration::from_secs(5)).await;
        }
    } else {
        return Err(Error::InvalidField("addresses"));
    }

    Ok(())
}

async fn process_transactions(context: &Context, transactions: &Value) -> Result<(), Error> {
    context.account.sync(None).await?;
    println!("{}", transactions);

    println!("{:?}", context.account.balance().await?);

    if let Some(transactions) = transactions.as_array() {
        for transaction in transactions {
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

                            let amount = if let Some(amount) = simple.get("amount") {
                                if let Some(amount) = amount.as_u64() {
                                    amount
                                } else {
                                    return Err(Error::InvalidField("amount"));
                                }
                            } else {
                                return Err(Error::MissingField("amount"));
                            };

                            let address = if let Some(address) = simple.get("address") {
                                if let Some(address) = address.as_u64() {
                                    address as usize
                                } else {
                                    return Err(Error::InvalidField("address"));
                                }
                            } else {
                                return Err(Error::MissingField("address"));
                            };

                            println!("{}", amount);
                            println!("{}", address);

                            let address = if let Some(address) = context.account.addresses().await?.get(address) {
                                address.address().as_ref().clone()
                            } else {
                                return Err(Error::InvalidField("address"));
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

            let transaction = context.account.send(outputs, None).await?;

            println!("{:?}", transaction);
        }
    } else {
        return Err(Error::InvalidField("transactions"));
    }

    Ok(())
}

fn process_tests(_context: &Context, tests: &Value) -> Result<(), Error> {
    println!("{}", tests);

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
        process_tests(context, tests)?;
    }

    Ok(())
}

async fn account_manager() -> Result<AccountManager, Error> {
    let mnemonic = "pumpkin actual foster argue normal dizzy sheriff action license hover fossil pink ancient company toilet silver egg chief actress month family dose orange corn";
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(mnemonic)?;

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
    let account = account_manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;
    let protocol_parameters = account.client().get_protocol_parameters()?;
    let context = Context {
        _account_manager: account_manager,
        account,
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
