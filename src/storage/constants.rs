// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The default storage folder.
pub(crate) const DEFAULT_STORAGE_FOLDER: &str = "./storage";

/// The default stronghold storage file name.
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub(crate) const STRONGHOLD_FILENAME: &str = "wallet.stronghold";

/// The default RocksDB storage path.
pub(crate) const ROCKSDB_FOLDERNAME: &str = "walletdb";

pub(crate) const ACCOUNT_MANAGER_INDEXATION_KEY: &str = "iota-wallet-account-manager";

pub(crate) const ACCOUNTS_INDEXATION_KEY: &str = "iota-wallet-accounts";
pub(crate) const ACCOUNT_INDEXATION_KEY: &str = "iota-wallet-account-";

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
// Key to store the first address in the db so it can be used to verify that new accounts use the same mnemonic
pub(crate) const FIRST_LEDGER_ADDRESS_KEY: &str = "FIRST_LEDGER_ADDRESS";
