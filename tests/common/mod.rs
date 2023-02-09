// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::redundant_pub_crate)]

mod constants;

use iota_client::{constants::SHIMMER_COIN_TYPE, request_funds_from_faucet, Client};
use iota_wallet::{
    account::AccountHandle,
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

pub use self::constants::*;

/// It creates a new account manager with a mnemonic secret manager, a client options object,
/// SHIMMER_COIN_TYPE, and a storage path
///
/// Arguments:
///
/// * `storage_path`: The path to the directory where the account manager will store its data.
/// * `mnemonic`: The mnemonic phrase that you want to use to generate the account. Defaults to a random one.
/// * `node`: The node to connect to. Defaults to `constants::NODE_LOCAL`
///
/// Returns:
///
/// An AccountManager
#[allow(dead_code, unused_variables)]
pub(crate) async fn make_manager(
    storage_path: &str,
    mnemonic: Option<&str>,
    node: Option<&str>,
) -> Result<AccountManager> {
    let client_options = ClientOptions::new().with_node(node.unwrap_or(NODE_LOCAL))?;
    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(mnemonic.unwrap_or(&Client::generate_mnemonic().unwrap()))?;

    #[allow(unused_mut)]
    let mut account_manager_builder = AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE);
    #[cfg(feature = "storage")]
    {
        account_manager_builder = account_manager_builder.with_storage_path(storage_path);
    }

    account_manager_builder.finish().await
}

/// Create `amount` new accounts, request funds from the faucet and sync the accounts afterwards until the faucet output
/// is available. Returns the new accounts.
#[allow(dead_code)]
pub(crate) async fn create_accounts_with_funds(
    account_manager: &AccountManager,
    amount: usize,
) -> Result<Vec<AccountHandle>> {
    let mut new_accounts = Vec::new();
    'accounts: for _ in 0..amount {
        let account = account_manager.create_account().finish().await?;
        request_funds_from_faucet(FAUCET_URL, &account.addresses().await?[0].address().to_bech32()).await?;

        // Continue only after funds are received
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let balance = account.sync(None).await?;
            if balance.base_coin.available > 0 {
                new_accounts.push(account);
                continue 'accounts;
            }
        }
        panic!("Faucet no longer wants to hand over coins");
    }

    Ok(new_accounts)
}

#[allow(dead_code)]
pub(crate) fn setup(path: &str) -> Result<()> {
    std::fs::remove_dir_all(path).unwrap_or(());
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn tear_down(path: &str) -> Result<()> {
    std::fs::remove_dir_all(path).unwrap_or(());
    Ok(())
}
