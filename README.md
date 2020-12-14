# IOTA Wallet Library

[![status](https://img.shields.io/badge/Status-Alpha-yellow.svg)](https://github.com/iotaledger/wallet.rs)

## Introduction

The wallet library is a stateful package with a standardised interface for developers to build applications involving IOTA value transactions.
It offers abstractions to handle IOTA payments and can optionally interact with [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs/) for seed handling, seed storage and state backup. See the full specification [here](https://github.com/iotaledger/wallet.rs/blob/master/specs/wallet-ENGINEERING-SPEC-0000.md).

## Warning

This library is in active development. The library targets the Chrysalis testnet and does not work with current IOTA mainnet.

## Prerequisites

To use the library, we recommend you update Rust to the latest stable version [`rustup update stable`](https://github.com/rust-lang/rustup.rs#keeping-rust-up-to-date). Nightly should be fine but there's a chance some changes are not compatible.

`no_std` is not supported currently, but we are working on it, and will provide it as a feature once the new implementation is ready.

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
  let mut manager = AccountManager::new().unwrap();
  manager.set_stronghold_password("my-password").unwrap();
  // now you can create accounts with `manager.create_account`, synchronize, send transfers, backup...
}
```

## API reference

You can read the [API reference](https://docs.rs/iota-wallet) here, or generate it yourself.

If you'd like to explore the implementation in more depth, the following command generates docs for the whole crate, including private modules:

```
cargo doc --document-private-items --no-deps --open
```

## Examples

You can see the examples in the [examples](examples/) directory and try them with:

```
cargo run --example # lists the available examples
cargo run --example transfer # execute the `transfer` example
```

## Joining the discussion

If you want to get involved in discussions about this library, or you're looking for support, go to the #clients-discussion channel on [Discord](https://discord.iota.org).
