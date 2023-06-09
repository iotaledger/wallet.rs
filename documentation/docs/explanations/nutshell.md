---
description: The `wallet.rs` library is written in Rust. You can also find three bindings written in Node.js, Python, and Java.
image: /img/logo/logo_dark.svg
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

# The Wallet Library in a Nutshell

The `wallet.rs` library uses an account model so you can create an account for each of your users. You can also take another approach and use one account and generate many addresses to link your users in your database.

Using the library allows you to assign a meaningful alias to each account. Users may also segregate their funds across multiple accounts or multiple addresses. It is up to a developer to choose a single-account approach or a multi-account approach.

The library is based on a [derivation for multiple accounts from a single seed](https://chrysalis.docs.iota.org/guides/dev_guide#addresskey-space). An account is a deterministic identifier from which multiple addresses can be further derived.

Below you will see the relationships between seeds, accounts, and addresses. A single seed can contain multiple accounts. Each account can also have multiple addresses which can be linked to users in your database.

![Seed, accounts and Addresses](/img/libraries/accounts_addresses.svg)
