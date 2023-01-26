// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::constants::SHIMMER_COIN_TYPE;
use iota_wallet::{
    account_manager::AccountManager,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    ClientOptions, Result,
};

mod constants;
pub use constants::*;

/// It creates a new account manager with a mnemonic secret manager, a client options object,
/// SHIMMER_COIN_TYPE, and a storage path
///
/// Arguments:
///
/// * `storage_path`: The path to the directory where the account manager will store its data.
/// * `mnemonic`: The mnemonic phrase that you want to use to generate the account. Defaults to
///   `constants::DEFAULT_MNEMONIC`
/// * `node`: The node to connect to. Defaults to `constants::NODE_LOCAL`
///
/// Returns:
///
/// An AccountManager
pub(crate) async fn make_manager(
    storage_path: &str,
    mnemonic: Option<&str>,
    node: Option<&str>,
) -> Result<AccountManager> {
    let client_options = ClientOptions::new().with_node(node.unwrap_or(NODE_LOCAL))?;
    let secret_manager = MnemonicSecretManager::try_from_mnemonic(mnemonic.unwrap_or(DEFAULT_MNEMONIC))?;

    AccountManager::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .with_storage_path(storage_path)
        .finish()
        .await
}

pub(crate) fn setup(path: &str) -> Result<()> {
    std::fs::remove_dir_all(path).unwrap_or(());
    Ok(())
}

pub(crate) fn tear_down(path: &str) -> Result<()> {
    std::fs::remove_dir_all(path).unwrap_or(());
    Ok(())
}
