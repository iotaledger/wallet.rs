# Welcome
This is the documentation for the official IOTA Wallet Library Software. It can be used to easily integrate an IOTA Wallet into your applications. You can read more about core principles behind IOTA client libraries in the following blog [post](https://blog.iota.org/the-new-iota-client-libraries-harder-better-faster-stronger/).

`Wallet.rs` is a general wallet library written in Rust. It is being utilized by our wallet software `Firefly` and other software components across IOTA ecosystem. `Wallet.rs` contains all the logic to safely build wallets or integrations that require value-based transfers (such as exchanges, pay-as-you-go systems, etc.). It includes account state management and backup, account creation, transferring tokens and much more. Needless to say, it is also based on our official `one-source-code-of-truth` [IOTA Rust library](https://github.com/iotaledger/iota.rs) and can be integrated with the [Stronghold enclave](https://blog.iota.org/iota-stronghold-6ce55d311d7c/) to achieve a maximum level of security.

> Using `stronghold` is a recommended approach to store account data using `wallet.rs`. The best security practices are integrated for free 

With the `wallet.rs` library, developers do not need to use a self-generated seed anymore. By default, the seed is created and stored in Stronghold encrypted at rest. It is not possible to extract the seed from Stronghold for security purposes. Stronghold uses encrypted snapshots that can easily be backed up and securely shared between devices. These snapshots are further secured with a password.

## IOTA 1.5 (Chrysalis) in a nutshell
All main concepts behind the IOTA Chrysalis are explained in detail at [Developer guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html).

Please, see a summary of changes in comparison to IOTA 1.0 at [Chrysalis documentation](https://chrysalis.docs.iota.org/guides/index.html).

## Warning
This library is in active development. The library targets the Chrysalis testnet and does not work with current IOTA mainnet.

## Testnet
To join the Chrysalis public testnet checkout [this link](https://blog.iota.org/chrysalis-phase-2-testnet-out-now/). More information about Chrysalis components is available at [documentation portal](https://chrysalis.docs.iota.org/).

## Joining the discussion
If you want to get involved in discussions about this library, or you're looking for support, go to the #clients-discussion channel on [Discord](https://discord.iota.org).

## What you will find here
This documentation has five paths:
1. The Overview, an detailed overview of the wallet library. 
2. Libraries, all available programming languages and their resources.
3. The Specification, detailed explanation requirements and functionality.
4. Contribute, how you can work on the wallet software.
5. Get in touch, join the community and become part of the X-Team!
