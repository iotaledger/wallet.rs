# IOTA Wallet Libraries

The `wallet.rs` library is writen in Rust.  You can also find two bindings writen in Node.js and Python:

- [Rust](rust/README.md)
- [Node.js](nodejs/README.md) 
- [Python](python/README.md)


## Getting Started
We recommended that you start your interactions with IOTA on a _testnet_ network. The _testnet_ will allow you to safely get acquainted with the `wallet.rs` library, without the risk of losing any funds if you make a mistake along the way. You can use this API load balancer: `api.lb-0.testnet.chrysalis2.com`  

A network explorer is available at [IOTA Tangle Explorer](https://explorer.iota.org/testnet).  You can use the network explorer to view transactions and data stored in the IOTA Tangle.    

In order to properly test value-based transactions on testnet network, you are going to need some tokens! You can get some testnet tokens through our [faucet](https://faucet.testnet.chrysalis2.com/).

## The Library in a Nutshell

`wallet.rs` uses an account model, so you can create an account for each of your users. You could also take another approach and use one account and generate many addresses, which you can link to your users in your database. 

The library allows users to assign a meaningful alias to each account. Users may also segregate their funds across multiple accounts or multiple addresses. It is up to a developer whether he chooses a single-account approach or multi-account approach.

The library is based on a [derivation for multiple accounts from a single seed](https://chrysalis.docs.iota.org/guides/dev_guide.html#addresskey-space). An account is simply a deterministic identifier from which multiple addresses can be further derived. 

The following image illustrates the relationships between seed, accounts and addresses.  A single seed can contain multiple accounts.  Each account can also have multiple addresses which can be linked to users in your database. 

![Seed, accounts and Addresses](../../static/img/libraries/accounts_addresses.svg)
