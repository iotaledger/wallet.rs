---
description: Getting started with the official IOTA Wallet Library Software Rust library.
image: /img/logo/wallet_light.png
keywords:
- Rust
- install
- cargo
- system environment variables
- getting started
---
# Getting Started with Rust

## Prerequisites

 _Rust_ and _Cargo_ are required to use wallet.rs. You can find installation instructions in the [Rust documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html).

We recommend you update _Rust_ to the [latest stable version](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). The nightly version should be fine, but there is a chance some changes are not compatible.

[_no_std_](https://docs.rust-embedded.org/book/intro/no-std.html) is not currently supported, but we are working on it, and we will provide it as a feature once the new implementation is ready.

### Dependencies

[_cmake_](https://cmake.org/documentation/) and [_openssl_](https://www.openssl.org/docs/) are required to run Rust. To run the build process successfully using Cargo, you may need install additional build tools onto your system. 

### Windows

You can download _cmake_ from the [official cmake website](https://cmake.org/download/).
You can install _openssl_ with [vcpkg](https://github.com/microsoft/vcpkg) or [chocolatey](https://chocolatey.org/).

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

You can install _cmake_ and _openssl_ with [_Homebrew_](https://docs.brew.sh/) by running the following commands:

```
brew install cmake
brew install openssl@1.1
# you may want to add this to your .zshrc or .bashrc since you'll need it to compile the crate
OPENSSL_ROOT_DIR=$(brew --prefix openssl@1.1)
```

### Linux

You can install _cmake_ and _openssl_ with your distro's package manager or download them from their websites. On Debian and Ubuntu, you will also need the [_build-essential_](https://packages.debian.org/sid/build-essential) package.

## Usage

### Dependencies

To use the library, add this to the _Cargo.toml_ file of your project:

```
[dependencies]
iota-wallet = { git = "https://github.com/iotaledger/wallet.rs", branch = "production" }
```

The example below requires asynchronious functionality. You can enable it by adding the following dependency:

```
tokio = { version = "1", features = ["full"] }
```

### Initialisation

To use the library, you first need to create an _AccountManager_ :

```rust
use iota_wallet::{account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let storage_folder: PathBuf = "./my-db".into();
    let manager = AccountManager::builder()
        .with_storage(&storage_folder, None)?
        .finish()
        .await?;
    manager.set_stronghold_password("password").await?;
    // If no mnemonic is provided, then the Stronghold file is the only way for a backup
    manager.store_mnemonic(SignerType::Stronghold, None).await?;
    let client_options = ClientOptionsBuilder::new()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")?
        .build()?;
    let account = manager
        .create_account(client_options)?
        .signer_type(SignerType::Stronghold)
        .initialise()
        .await?;
    let address = account.generate_address().await?;
    println!("Address: {}", address.address().to_bech32());
    Ok(())
}
```
