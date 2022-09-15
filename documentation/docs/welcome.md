---
description: Official IOTA Wallet Library Software which can be used to easily integrate an IOTA Wallet into your application 
image: /img/logo/wallet_light.png
keywords:
- requirements
- wallet
- software
- library
- rust
- python
- nodejs
- java
---
# Welcome

This is the documentation for the official IOTA Wallet Library Software. The documentation can be used to easily integrate an IOTA Wallet into your applications. You can read more about core principles behind IOTA client libraries in the following [blog post](https://blog.iota.org/the-new-iota-client-libraries-harder-better-faster-stronger/).

`wallet.rs` is a general wallet library written in Rust. It is being utilized by our wallet software `Firefly` and other software components across IOTA ecosystem. `wallet.rs` contains all the logic to safely build wallets or integrations that require value-based transfers (such as exchanges, pay-as-you-go systems, etc.). `wallet.rs` includes account state management and backup, account creation, transferring tokens and much more. Needless to say, it is also based on our official _one-source-code-of-truth_ [IOTA Rust library](https://github.com/iotaledger/wallet.rs) and can be integrated with the [Stronghold enclave](https://blog.iota.org/iota-stronghold-6ce55d311d7c/) to achieve a maximum level of security.

:::caution
Use Stronghold for secrets management. It integrates the best security practices and is open-source.
:::

`Stronghold` can store the encrypted seed at rest. It is not possible to extract the seed from `Stronghold` for security purposes. `Stronghold` uses encrypted snapshots that can easily be backed up and securely shared between devices. These snapshots are further secured with a password.

## Testnet

To join the public testnet checkout this [blog post](https://blog.shimmer.network/shimmer-beta-network-is-live). More information about Stardust components is available in the [tips repository](https://github.com/iotaledger/tips/pulls).

## Joining the Discussion

If you want to get involved in discussions about this library, or you're looking for support, go to the #wallet-library channel on [Discord](https://discord.iota.org).
