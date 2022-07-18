---
description: Getting started with the official IOTA Wallet Library Software Rust library.
image: /img/logo/wallet_light.png
keywords:
- Rust
- install
- cargo
- system environment variables
---
# Getting Started with Rust

## Prerequisites

 _Rust_ and _Cargo_ are required to use wallet.rs. You can find installation instructions in the [Rust documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html).

We recommend you update _Rust_ to the latest stable version [rustup update stable](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date).

### Dependencies

 [_cmake_](https://cmake.org/documentation/) and [_openssl_](https://www.openssl.org/docs/) are required. In order to run the build process successfully using Cargo you may need install additional build tools on your system. 

### Windows

 _cmake_ can be downloaded from the [official cmake website](https://cmake.org/download/).
 _openssl_ can be installed with [vcpkg](https://github.com/microsoft/vcpkg) or [chocolatey](https://chocolatey.org/).

- Installing _openssl_ with _vcpkg_ :

    ```
    ./vcpkg.exe install openssl:x64-windows
    ./vcpkg.exe integrate install
    # you may want to add this to the system environment variables since you'll need it to compile the crate
    set VCPKGRS_DYNAMIC=1
    ```

- Installing _openssl_ with _chocolatey_ :

    ```
    choco install openssl
    # you may need to set the OPENSSL_ROOT_DIR environment variable
    set OPENSSL_ROOT_DIR="C:\Program Files\OpenSSL-Win64"
    ```

### macOS

 _cmake_ and _openssl_ can be installed with [_Homebrew_](https://docs.brew.sh/) by running the following commands:

```
brew install cmake
brew install openssl@1.1
# you may want to add this to your .zshrc or .bashrc since you'll need it to compile the crate
OPENSSL_ROOT_DIR=$(brew --prefix openssl@1.1)
```

### Linux

You can install _cmake_ and _openssl_ with your distro's package manager or download from their websites. On Debian and Ubuntu you will also need the [_build-essential_](https://packages.debian.org/sid/build-essential) package.

## Usage

To use the library, add this to your _Cargo.toml_ :

```
[dependencies]
iota-wallet = { git = "https://github.com/iotaledger/wallet.rs", branch = "develop" }
```

### Initialisation

In order to use the library, you first need to create an _AccountManager_ :

```rust
use std::path::PathBuf;

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Shouldn't be hardcoded in production
    // mnemonic can be generated with `manager.generate_mnemonic()?` and will be the only way to recover your funds if
    // you loose the stronghold file/password, so be sure to save it securely
    let nonsecure_use_of_development_mnemonic = "endorse answer radar about source reunion marriage tag sausage weekend frost daring base attack because joke dream slender leisure group reason prepare broken river".to_string();
    let stronghold_password = "some_hopefully_secure_password";

    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
        .password(&stronghold_password)
        .snapshot_path(PathBuf::from("wallet.stronghold"))
        .build();

    // The mnemonic only needs to be stored the first time
    secret_manager
        .store_mnemonic(nonsecure_use_of_development_mnemonic)
        .await?;

    // Create the account manager with the secret_manager and client options
    let client_options = ClientOptions::new().with_node("http://localhost:14265")?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Create a new account, this will automatically generate an address
    let account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    println!(
        "Generated a new account with addresses {:?}",
        account.list_addresses().await?
    );

    Ok(())
}
```
