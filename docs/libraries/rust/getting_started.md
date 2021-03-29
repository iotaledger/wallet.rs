# Getting Started with Rust

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