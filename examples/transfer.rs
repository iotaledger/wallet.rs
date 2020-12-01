// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, message::Transfer,
};

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let mut manager = AccountManager::new().unwrap();
    manager.set_stronghold_password("password").unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")?.build();
    let account = manager
        .create_account(client_options)
        .alias("alias")
        .initialise()?;

    // we need to synchronize with the Tangle first
    let sync_accounts = manager.sync_accounts().await?;
    let sync_account = sync_accounts.first().unwrap();
    sync_account
        .transfer(Transfer::new(
            account.latest_address().unwrap().address().clone(),
            150,
        ))
        .await?;

    Ok(())
}
