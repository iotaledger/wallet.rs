---
description: The `wallet.rs` library is written in Rust. You can also find three bindings written in Node.js, Python, and Java.
image: /img/logo/wallet_light.png
keywords:
- bindings
- library
- rust
- python
- java
- node.js
- account
- multiple
- explanation 
---
# IOTA Wallet Libraries

The `wallet.rs` library is written in Rust. You can also find bindings written in Node.js, Python, and Java:

- [Rust](rust/getting_started.md).
- [Node.js](nodejs/getting_started.md).
- [Python](python/getting_started.md).
- [Java](java/getting_started.md).

## Getting Started

We recommended that you start your interactions with IOTA on the [devnet](https://wiki.iota.org/learn/networks/testnets#iota-20-decentralized-devnet). The _devnet_ will allow you to safely get acquainted with the `wallet.rs` library, without the risk of losing any funds if you make a mistake along the way. You can use this API load balancer: `api.lb-0.h.chrysalis-devnet.iota.cafe`.  

You can also use the network explorer that is available at [IOTA Tangle Explorer](https://explorer.iota.org/devnet). You can use the network explorer to view transactions and data stored in the IOTA Tangle.    

To properly test value-based transactions on the devnet, you will need some tokens. You can get some devnet tokens using the [faucet](https://faucet.chrysalis-devnet.iota.cafe/).

## The Library in a Nutshell

`wallet.rs` uses an account model so you can create an account for each of your users. You can also take another approach and use one account and generate many addresses to link your users in your database.

Using the library allows you to assign a meaningful alias to each account. Users may also segregate their funds across multiple accounts or multiple addresses. It is up to a developer to choose a single-account approach or a multi-account approach.

The library is based on a [derivation for multiple accounts from a single seed](https://chrysalis.docs.iota.org/guides/dev_guide#addresskey-space). An account is a deterministic identifier from which multiple addresses can be further derived.

Below you will see the relationships between seeds, accounts, and addresses. A single seed can contain multiple accounts. Each account can also have multiple addresses which can be linked to users in your database.

![Seed, accounts and Addresses](/img/libraries/accounts_addresses.svg)
