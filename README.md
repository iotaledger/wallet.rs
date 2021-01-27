# IOTA Wallet Library

[![status](https://img.shields.io/badge/Status-Alpha-yellow.svg)](https://github.com/iotaledger/wallet.rs)

## Introduction

The wallet library is a stateful package with a standardised interface for developers to build applications involving IOTA value transactions.
It offers abstractions to handle IOTA payments and can optionally interact with [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs/) for seed handling, seed storage and state backup. Alternatively you can use the EnvMnemonic SignerType and a SQLite database. See the full specification [here](https://github.com/iotaledger/wallet.rs/blob/master/specs/wallet-ENGINEERING-SPEC-0000.md).

## Warning

This library is in active development. The library targets the Chrysalis testnet and does not work with current IOTA mainnet.

The Stronghold integration is not complete. We recommend you use SQLite and the EnvMnemonic SignerType (allowing you to store your mnemonic as an environment variable).

## Prerequisites

`Rust` and `Cargo` are required. Install them [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

We recommend you update Rust to the latest stable version [`rustup update stable`](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). Nightly should be fine but there's a chance some changes are not compatible.

`no_std` is not supported currently, but we are working on it, and will provide it as a feature once the new implementation is ready.

### Dependencies

`cmake` and `openssl` are required. In order to run the build process succesfully using Cargo you might need install additional build tools on your system. 

### Windows

`cmake` can be downloaded on the [official website](https://cmake.org/download/) and `openssl` can be installed with [vcpkg](https://github.com/microsoft/vcpkg) or [chocolatey](https://chocolatey.org/).

- Installing `openssl` with `vcpkg`:

```
$ ./vcpkg.exe install openssl:x64-windows
$ ./vcpkg.exe integrate install
# you may want to add this to the system environment variables since you'll need it to compile the crate
$ set VCPKGRS_DYNAMIC=1
```

- Installing `openssl` with `chocolatey`:

```
$ choco install openssl
# you may need to set the OPENSSL_ROOT_DIR environment variable
$ set OPENSSL_ROOT_DIR="C:\Program Files\OpenSSL-Win64"
```

### macOS

`cmake` and `openssl` can be installed with `Homebrew`:

```
$ brew install cmake
$ brew install openssl@1.1
# you may want to add this to your .zshrc or .bashrc since you'll need it to compile the crate
$ OPENSSL_ROOT_DIR=$(brew --prefix openssl@1.1)
```

### Linux

Install `cmake` and `openssl` with your distro's package manager or download from their websites. On Debian and Ubuntu you will also need `build-essential`.

## Usage

To use the library, add this to your `Cargo.toml`:

```
[dependencies]
iota-wallet = { git = "https://github.com/iotaledger/wallet.rs" }
```

### Initialisation

In order to use the library you first need to create an `AccountManager`:

```rust
use iota_wallet::{
    account_manager::AccountManager, client::ClientOptionsBuilder, signing::SignerType,
    storage::sqlite::SqliteStorageAdapter,
};
use std::path::PathBuf;
#[tokio::main]
async fn main() -> iota_wallet::Result<()> {
    let storage_folder: PathBuf = "./my-db".into();
    let manager =
        AccountManager::with_storage_adapter(&storage_folder, SqliteStorageAdapter::new(&storage_folder, "accounts")?)?;
    let client_options = ClientOptionsBuilder::node("http://api.lb-0.testnet.chrysalis2.com")?.build();
    let account = manager
        .create_account(client_options)
        .signer_type(SignerType::EnvMnemonic)
        .initialise()?;
    Ok(())
}
```

## API reference

If you'd like to explore the implementation in more depth, the following command generates docs for the whole crate:

```
cargo doc --document-private-items --no-deps --open
```

## Other Examples

You can see the examples in the [examples](examples/) directory and try them with:

```
cargo run --example # lists the available examples
cargo run --example transfer # execute the `transfer` example
```

## Joining the discussion

If you want to get involved in discussions about this library, or you're looking for support, go to the #clients-discussion channel on [Discord](https://discord.iota.org).
