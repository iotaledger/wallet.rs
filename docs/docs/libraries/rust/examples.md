# Examples
You can see the examples in the library's [examples directory](https://github.com/iotaledger/wallet.rs/tree/dev/examples).
You can list all available examples by running the following command:
```
cargo run --example # lists the available examples
```
To run an example, you can use the following command, replacing `transfer` with the desired example:
```
cargo run --example transfer # execute the `transfer` example
```

## Backup and Restore Example

1. Create an account manager and set a password:
```rust
let mut manager = AccountManager::builder().finish().await.unwrap();

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