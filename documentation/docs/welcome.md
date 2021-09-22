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

`wallet.rs` is a general wallet library written in Rust. It is being utilized by our wallet software `Firefly` and other software components across IOTA ecosystem. `wallet.rs` contains all the logic to safely build wallets or integrations that require value-based transfers (such as exchanges, pay-as-you-go systems, etc.). `wallet.rs` includes account state management and backup, account creation, transferring tokens and much more. Needless to say, it is also based on our official _one-source-code-of-truth_ [IOTA Rust library](https://github.com/iotaledger/iota.rs) and can be integrated with the [Stronghold enclave](https://blog.iota.org/iota-stronghold-6ce55d311d7c/) to achieve a maximum level of security.

:::caution
Use Stronghold to store account data of the `wallet.rs`. It integrates the best security practices and is open-source.
:::

With the `wallet.rs` library, developers do not need to use a self-generated seed anymore. By default, `Stronghold` will create and store the encrypted seed at rest. It is not possible to extract the seed from `Stronghold` for security purposes. `Stronghold` uses encrypted snapshots that can easily be backed up and securely shared between devices. These snapshots are further secured with a password.

## IOTA 1.5 (Chrysalis) in a Nutshell
The [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide) explains all the main concepts behind the IOTA Chrysalis in detail.

Please, see a summary of changes in comparison to IOTA 1.0 at [Chrysalis documentation](https://chrysalis.docs.iota.org/guides).

## Testnet
To join the Chrysalis public testnet checkout this [blog post](https://blog.iota.org/chrysalis-phase-2-testnet-out-now/). More information about Chrysalis components is available at the [Chrysalis documentation portal](https://chrysalis.docs.iota.org/).

## Joining the Discussion
If you want to get involved in discussions about this library, or you're looking for support, go to the #clients-discussion channel on [Discord](https://discord.iota.org).

## What You Will Find Here
This documentation has four paths:
1. The [Overview](overview.md): a detailed overview of the wallet library. 
2. [Libraries](libraries/overview.md): all available programming languages and their resources.
3. The [Specification](specification.md): a detailed explanation requirements and functionality.
4. [Contribute](contribute.md): how you can work on the wallet software, get in touch, join the community and become part of the X-Team!
