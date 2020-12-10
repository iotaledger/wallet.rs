// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder};

fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::new().unwrap();
    manager.set_stronghold_password("password").unwrap();

    // first we'll create an example account
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    let account = manager.create_account(client_options).alias("alias").initialise()?;
    let id = account.id();

    // backup the stored accounts to ./backup/${backup_name}
    let backup_path = manager.backup("./backup")?;

    // delete the account on the current storage
    manager.remove_account(&id)?;

    // import the accounts from the backup and assert that it's the same
    manager.import_accounts(backup_path)?;
    let imported_account = manager.get_account(id)?;
    assert_eq!(account, imported_account);

    Ok(())
}
