# Overview

The aim of this wallet is to have an improved version of [the current wallet](https://github.com/iotaledger/wallet.rs), by redesigning a few parts (most important, move away from messages), to have it cleaner and better mainainable for the future.

The wallet should consist of a core part which provides needed functionallity to generate addresses, get balance and send transactions, but which isn't affected by any extended functionallity.
Via Rust features extendend functionallity will be available later, like having a database for the state, different signer types (Stronghold, Ledger), being able to store events and high level functions like internal_transfers, which can be used to send a transfer from one account to another.

## Account Manager

When interacting with the wallet, one first needs to build the account manager, which is used to create and get accounts. One account manager can hold many accounts, but they should all share the same signer type with the same seed/mnemonic.
It also manages the background syncing by calling the syncing function for each account.

## Account

An account is used to generate addresses and create transactions with available funds.
For the interaction with the Tangle the [iota_client](https://github.com/iotaledger/iota.rs/) is used.

## Signer

A signer is used to generate adddresses and sign transactions, it will be used by an account.

Possible types are [Stronghold](https://github.com/iotaledger/stronghold.rs/), Ledger (only supports SignatureLockedSingle outputs), LedgerSimulator, Mnemonic and custom implementations could be added.

## Events

With the event feature, the wallet emits different events to which a user can listen from the account manager. If no events types are specified all events are received

pub enum WalletEventType {
    BalanceChange,
    TransactionInclusion,
    TransferProgress,
    ConsolidationRequired,
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    LedgerAddressGeneration,
}

## Actor (should also be a Rust feature?)

We want to provide an easy interface for [Firefly](https://github.com/iotaledger/firefly/) and bindings to call the functions from the wallet without binding each function individually.

## Docs

With cargo watch `cargo watch -x "doc --document-private-items --no-deps --all-features"`.

Build the rust docs with `cargo doc --document-private-items --no-deps --open --all-features --release` and copy the result to the docs folder with `cp target/doc/*.*  ./docs` and `cp -r target/doc/wallet_core  ./docs/wallet_core`.
