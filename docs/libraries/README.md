# IOTA Wallet Libraries

There are currently available the following official bindings to `wallet.rs`:
- [Rust](./rust/) 
- [Node.js](./nodejs/) 
- [Python](./python/)

## Getting Started
It is a recommended approach to start your interactions with IOTA on a `testnet` network.  API load balancer: api.lb-0.testnet.chrysalis2.com  

Network explorer is available at [IOTA Tangle Explorer](https://explorer.iota.org/testnet).

In order to properly test value-based transactions on testnet network, you are going to need some tokens! You can get some testnet tokens using the [faucet](https://faucet.testnet.chrysalis2.com/).

## The library in a nutshell
In the `wallet.rs`, we use an account model, so you can create an account for each of your users. The other approach would be to use one account and generate many addresses, which you can link to your users in your database. The library allows users to assign a meaningful alias to each account. It also leaves the choice to users if they want to segregate their funds across multiple accounts or multiple addresses. So basically it is up to a developer to decide whether `single-account approach` or `multi-account approach` is chosen. The library provides a support to any of the scenarios.

The library is based on a [derivation for multiple accounts from a single seed](https://chrysalis.docs.iota.org/guides/dev_guide.html#addresskey-space). An account is simply a deterministic identifier from which multiple addresses can be further derived. The following illustrates the relationships between seed, accounts and addresses:

![accounts](accounts_addresses.svg)