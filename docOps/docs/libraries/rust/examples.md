# Examples
You can see the examples in the library's [examples directory](https://github.com/iotaledger/wallet.rs/tree/dev/examples).
You can list all available examples by running the following command:
```
cargo run --example # lists the available examples
```
To run an example, you can use the following command, replacing _transfer_ with the desired example:
```
cargo run --example transfer # execute the `transfer` example
```

## Backup and Restore Example

1. Create an account manager and set a password:
```rust
let manager = AccountManager::builder().finish().await.unwrap();

manager.set_stronghold_password("password").await.unwrap();
manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

```

2. Create your account:

```rust
let client_options = ClientOptionsBuilder::new()
    .with_node("https://api.lb-0.testnet.chrysalis2.com")?
    .build()
    .unwrap();
let account_handle = manager
    .create_account(client_options)?
    .alias("alias")
    .initialise()
    .await?;
let id = account_handle.id().await;

```

3. You can secure your account in a backup file:
```rust
// backup the stored accounts to ./backup/${backup_name}
let backup_path = manager.backup("./backup").await?;

```


4. You can import the backup later, or in another application using the following snippet:
```rust
manager.import_accounts(backup_path, "password").await?;

let imported_account_handle = manager.get_account(&id).await?;

let account = account_handle.read().await;
let imported_account = imported_account_handle.read().await;

```

That's it! You can now backup and restore your account!

You can see the full code for the example in the [`Wallet.rs` repository](https://github.com/iotaledger/wallet.rs/blob/develop/examples/backup_and_restore.rs)


## Transfer Example:
You use the following example to generate an account, and transfer funds. 

```rust
// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, message::Transfer, signing::SignerType,
};
use std::num::NonZeroU64;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let manager = AccountManager::builder().finish().await.unwrap();
    manager.set_stronghold_password("password").await.unwrap();
    manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.testnet.chrysalis2.com")?
        .build()
        .unwrap();
    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    let address = account.generate_address().await?;
    println!(
        "Send iotas from the faucet to {} and press enter after the transaction got confirmed",
        address.address().to_bech32()
    );
    let mut message = String::new();
    std::io::stdin().read_line(&mut message).unwrap();
    println!("Sending transfer...");
    let message = account
        .transfer(
            Transfer::builder(
                account.latest_address().await.address().clone(),
                NonZeroU64::new(1500000).unwrap(),
            )
            .finish(),
        )
        .await?;
    println!("Message sent: {}", message.id());

    Ok(())
}

```

## Events example:
`Wallet.rs` library is able to listen to several supported event. As soon as the event occurs, a provided callback will be triggered.

You can use the following example to fetch an existing _Account_ and listen to transaction events related to that _Account_ :
```rust
// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example event --release

use iota_wallet::{
    account_manager::AccountManager, address::Address, client::ClientOptionsBuilder, event::on_balance_change,
    message::MessageId, signing::SignerType, Result,
};
use serde::Deserialize;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = AccountManager::builder().finish().await?;
    manager.set_stronghold_password("password").await?;
    manager.store_mnemonic(SignerType::Stronghold, None).await?;

    // first we'll create an example account and store it
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.testnet.chrysalis2.com")?
        .build()?;

    let account = manager
        .create_account(client_options)?
        .alias("alias")
        .initialise()
        .await?;

    // Possible events are: on_balance_change, on_broadcast, on_confirmation_state_change, on_error,
    // on_migration_progress, on_new_transaction, on_reattachment, on_stronghold_status_change,
    // on_transfer_progress,
    on_balance_change(move |event| {
        println!("BalanceEvent: {:?}", event);
        println!("Press enter to exit");
    })
    .await;

    let address = account.generate_address().await?;
    println!("Requesting funds from the faucet to {}", address.address().to_bech32());
    get_funds(&address).await?;

    // Wait for event before exit
    let mut exit = String::new();
    std::io::stdin().read_line(&mut exit).unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct FaucetMessageResponse {
    id: String,
}

#[derive(Deserialize)]
struct FaucetResponse {
    data: FaucetMessageResponse,
}

async fn get_funds(address: &Address) -> Result<MessageId> {
    // use the faucet to get funds on the address
    let response = reqwest::get(&format!(
        "https://faucet.testnet.chrysalis2.com/api?address={}",
        address.address().to_bech32()
    ))
    .await
    .unwrap()
    .json::<FaucetResponse>()
    .await
    .unwrap();
    let faucet_message_id = MessageId::from_str(&response.data.id)?;

    println!("Got funds from faucet, message id: {:?}", faucet_message_id);

    Ok(faucet_message_id)
}

```

## Logger example:

```rust
// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example logger --release

use iota_client::common::logger::{logger_init, LoggerConfig, LoggerOutputConfigBuilder};
use iota_wallet::account_manager::AccountManager;
use log::LevelFilter;
use std::time::Instant;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    // Generates a wallet.log file with logs for debugging
    let output_config = LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .level_filter(LevelFilter::Debug);
    let config = LoggerConfig::build().with_output(output_config).finish();
    logger_init(config).unwrap();

    let manager = AccountManager::builder()
        .with_storage("./backup", None)?
        .with_skip_polling()
        .finish()
        .await?;
    manager.set_stronghold_password("password").await?;

    let account = manager.get_account("Alice").await?;

    let now = Instant::now();
    account.sync().await.execute().await?;
    println!("Syncing took: {:.2?}", now.elapsed());

    println!("Balance: {:?}", account.balance().await?);

    let addresses = account.list_unspent_addresses().await?;
    println!("Addresses: {}", addresses.len());

    let address = account.generate_address().await?;
    println!("Generated a new address: {:?}", address);

    Ok(())
}
```
