---
description: The official IOTA Wallet Library Software can be used to integrate an IOTA Wallet into your application. 
image: /img/logo/logo.svg
keywords:
- requirements
- wallet
- software
- library
- rust
- python
- nodejs
- java
- reference
---

# Welcome

Welcome to the official IOTA Wallet Library Software documentation. You can use our documentation to help streamline integrating an IOTA Wallet into your applications. If you want to learn more about the IOTA client libraries, you can check out our [blog post](https://blog.iota.org/the-new-iota-client-libraries-harder-better-faster-stronger/) on how they work.

## wallet.rs

`wallet.rs` is a general wallet library written in Rust. It is currently utilized by our wallet software, [Firefly](https://firefly.iota.org/), and other software components across the IOTA ecosystem. 

`wallet.rs` contains all of the specs to safely build wallets or integrations that require value-based transfers, such as exchanges and pay-as-you-go systems. Additionally, amongst other features, `wallet.rs` includes account state management and backup, account creation, and transferring tokens. `wallets.rs` is also based on our official _one-source-code-of-truth_ [IOTA Rust library](https://github.com/iotaledger/iota.rs) and can be integrated with the [Stronghold enclave](https://blog.iota.org/iota-stronghold-6ce55d311d7c/) to achieve a maximum level of security.

:::note

You can use Stronghold to store account data of the `wallet.rs`. It integrates the best security practices and is open-source.

:::

### Stronghold and wallet.rs

With the `wallet.rs` library, developers no longer need to use a self-generated seed anymore. By default, the security of `Stronghold` creates and stores the encrypted seed, at rest. Additionally, it is not possible to extract the seed from `Stronghold` as a security measure. 

`Stronghold` also uses encrypted snapshots that can easily be backed up and securely shared between devices. These snapshots are further secured with a password.

## IOTA 1.5 (Chrysalis) in a Nutshell

For some background, you can checkout our [Developer Guide to Chrysalis](https://wiki.iota.org/introduction/explanations/update/what_is_chrysalis/) which explains all of the main concepts behind the IOTA Chrysalis in detail.

You can also read our [Chrysalis documentation](https://wiki.iota.org/introduction/welcome/) to learn the history of versions between IOTA 1.0 and Chrysalis.

## Testnet

To join the Chrysalis public devnet, check out our [blog post](https://blog.iota.org/chrysalis-phase-2-testnet-out-now/). You can also learn more about the different Chrysalis components in our [Chrysalis documentation portal](https://wiki.iota.org/introduction/welcome/).