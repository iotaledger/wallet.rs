# Examples
You can see the examples in the [examples](examples/) directory and try them with:

```
cargo run --example # lists the available examples
cargo run --example transfer # execute the `transfer` example
```

## Backup and restore example

Create an account manager and set an password:
```rust
let manager = AccountManager::builder().finish().await.unwrap();

manager.set_stronghold_password("password").await.unwrap();
manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

```

Create your account:

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

Now you can secure your account in a backup file:
```rust
// backup the stored accounts to ./backup/${backup_name}
let backup_path = manager.backup("./backup").await?;

```


You can import the backup later or by another application like here:
```rust
manager.import_accounts(backup_path, "password").await?;

let imported_account_handle = manager.get_account(&id).await?;

let account = account_handle.read().await;
let imported_account = imported_account_handle.read().await;

```

That's it! Now you know, how do backup and restore your account!

See the full example [here](https://github.com/iotaledger/wallet.rs/blob/develop/examples/backup_and_restore.rs)