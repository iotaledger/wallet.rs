# IOTA Wallet Library

[![status](https://img.shields.io/badge/Status-Alpha-yellow.svg)](https://github.com/iotaledger/wallet.rs)

## Introduction

The wallet library is a stateful package with a standardised interface for developers to build applications involving IOTA value transactions.
It offers abstractions to handle IOTA payments, seed security and state backup through [stronghold](https://github.com/iotaledger/stronghold.rs/) and account privacy.

## Prerequisites

To use the library, we recommend update your Rust to latest stable version [`rustup update stable`](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). Nightly should be fine but you are expected some changes might not be compatable.

`no_std` is not supported currently, but we are working on it in [bee](https://github.com/iotaledger/bee), and will provide it as feature once new library implementation is ready.

## Usage

To use the library, add this to your `Cargo.toml`:

```
[dependencies]
iota-wallet = { git = "https://github.com/iotaledger/wallet.rs" }
```

### Initialisation

In order to use the library you first need to create an `AccountManager`:

```rust
use iota_wallet::account_manager::AccountManager;
fn main() {
  let manager = AccountManager::new();
  // now you can create accounts with `manager.create_account`, synchronize, send transfers, backup...
}
```

## API reference

You can read the [API reference](https://docs.rs/iota-wallet) here, or generate them on your own.

If you'd like to explore the implementation in more depth, the following command generates docs for the whole crate, including private modules:

```
cargo doc --document-private-items --no-deps --open
```

## Examples

You can see the examples in [examples](examples/) directory and try them with:

```
cargo run --example # lists the available examples
cargo run --example transfer # execute the `transfer` example
```

## License

The MIT license can be found [here](LICENSE).
