// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod error;

use iota_wallet::{
    account::AccountHandle,
    account_manager::AccountManager,
    iota_client::{
        constants::SHIMMER_COIN_TYPE,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    ClientOptions,
};
use serde_json::Value;
use tokio::fs;

use self::error::Error;

struct Context {
    _account_manager: AccountManager,
    account: AccountHandle,
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
                    return Err(Error::InvalidField("addresses"));
                }
            }

            let addresses = context.account.generate_addresses(amounts.len() as u32, None).await?;

            println!("{:?}", addresses);
        }
    }

    Ok(())
}

fn process_transactions(_context: &Context, transactions: &Value) -> Result<(), Error> {
    println!("{}", transactions);

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
        process_transactions(context, transactions)?;
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
    let context = Context {
        _account_manager: account_manager,
        account,
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
