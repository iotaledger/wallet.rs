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

We recommend you update _Rust_ to the latest stable version [rustup update stable](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). The nightly version should be fine, but there is a chance some changes are not compatible.

 [_no_std_](https://docs.rust-embedded.org/book/intro/no-std.html) is not currently supported, but we are working on it, and we will provide it as a feature once the new implementation is ready.

### Dependencies

 [_cmake_](https://cmake.org/documentation/) and [_openssl_](https://www.openssl.org/docs/) are required. In order to run the build process successfully using Cargo you may need install additional build tools on your system. 

### Windows

 _cmake_ can be downloaded from the [official cmake website](https://cmake.org/download/).
 _openssl_ can be installed with [vcpkg](https://github.com/microsoft/vcpkg) or [chocolatey](https://chocolatey.org/).

- Installing _openssl_ with _vcpkg_ :

    ```
    $ ./vcpkg.exe install openssl:x64-windows
    $ ./vcpkg.exe integrate install
    # you may want to add this to the system environment variables since you'll need it to compile the crate
    $ set VCPKGRS_DYNAMIC=1
    ```

- Installing _openssl_ with _chocolatey_ :

    ```
    $ choco install openssl
    # you may need to set the OPENSSL_ROOT_DIR environment variable
    $ set OPENSSL_ROOT_DIR="C:\Program Files\OpenSSL-Win64"
    ```

### macOS

 _cmake_ and _openssl_ can be installed with [_Homebrew_](https://docs.brew.sh/) by running the following commands:

```
$ brew install cmake
$ brew install openssl@1.1
# you may want to add this to your .zshrc or .bashrc since you'll need it to compile the crate
$ OPENSSL_ROOT_DIR=$(brew --prefix openssl@1.1)
```

### Linux

You can install _cmake_ and _openssl_ with your distro's package manager or download from their websites. On Debian and Ubuntu you will also need the [_build-essential_](https://packages.debian.org/sid/build-essential) package.

## Usage

To use the library, add this to your _Cargo.toml_ :

```
[dependencies]
iota-wallet = { git = "https://github.com/iotaledger/wallet.rs", branch = "dev" }
```

### Initialisation

In order to use the library, you first need to create an _AccountManager_ :

```rust
use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let storage_folder: PathBuf = "./my-db".into();
    let manager =
        AccountManager::builder()
            .with_storage(&storage_folder, None)
            .finish()
            .await?;
    let client_options = ClientOptionsBuilder::new().with_node("http://api.lb-0.testnet.chrysalis2.com")?.build();
    let account = manager
        .create_account(client_options)
        .signer_type(SignerType::EnvMnemonic)
        .initialise()
        .await?;
    Ok(())
}
```