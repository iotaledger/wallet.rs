---
description: The `wallet.rs` library is written in Rust. You can also find three bindings written in Node.js, Python and Java.
image: /img/logo/wallet_light.png
keywords:
- bindings
- library
- rust
- python
- node.js
- account
- multiple
---
# IOTA Wallet Libraries

The `wallet.rs` library is written in Rust.  You can also find a binding written in Node.js.

- [Rust](rust/getting_started.md)
- [Node.js](nodejs/getting_started.md) 


## Getting Started

We recommended that you start your interactions with IOTA on a _testnet_ network. The _testnet_ will allow you to safely get acquainted with the `wallet.rs` library, without the risk of losing any funds if you make a mistake along the way. You can use this API load balancer: `https://api.testnet.shimmer.network`  

A network explorer is available at [IOTA Tangle Explorer](TODO: set correct explorer link).  You can use the network explorer to view transactions and data stored in the IOTA Tangle.    

In order to properly test value-based transactions on testnet network, you are going to need some tokens! You can get some testnet tokens through our [faucet](https://faucet.testnet.shimmer.network).

## The Library in a Nutshell

`wallet.rs` uses an account model, so you can create an account for each of your users. You could also take another approach and use one account and generate many addresses, which you can link to your users in your database. 

The library allows users to assign a meaningful alias to each account. Users may also segregate their funds across multiple accounts or multiple addresses. It is up to a developer whether he chooses a single-account approach or multi-account approach.

The library is based on a [derivation for multiple accounts from a single seed](https://chrysalis.docs.iota.org/guides/dev_guide#addresskey-space). An account is simply a deterministic identifier from which multiple addresses can be further derived. 

The following image illustrates the relationships between seed, accounts and addresses.  A single seed can contain multiple accounts.  Each account can also have multiple addresses which can be linked to users in your database. 

![Seed, accounts and Addresses](/img/libraries/accounts_addresses.svg)
