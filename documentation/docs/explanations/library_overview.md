---
description: 'The wallet library is a stateful package with a standardized interface for developers to build applications
involving IOTA value transactions.'
image: /img/logo/wallet_light.png
keywords:

- layered overview
- high level
- low level
- stronghold
- value transactions

---

# Library Overview

The wallet library is a stateful package with a standardized interface for developers to build applications involving
IOTA value transactions. It provides abstractions to handle IOTA payments and can optionally interact
with [IOTA Stronghold enclave](https://github.com/iotaledger/stronghold.rs/) for seed handling, seed storage and state
backup.

## High Level Layered Overview

![High Level Layered Overview](/img/overview/iota_layers_overview.svg)

## The Library in a Nutshell

`wallet.rs` uses an account model, so you can create an account for each of your users. You could also take another
approach and use one account and generate many addresses, which you can link to your users in your database.

The library allows users to assign a meaningful alias to each account. Users may also segregate their funds across
multiple accounts or multiple addresses. It is up to a developer whether he chooses a single-account approach or
multi-account approach.

The library is based on
a [derivation for multiple accounts from a single seed](https://wiki.iota.org/introduction/reference/details/#addresskey-space). 
An account is simply a deterministic identifier from which multiple addresses can be further derived.

The following image illustrates the relationships between seed, accounts and addresses. A single seed can contain
multiple accounts. Each account can also have multiple addresses which can be linked to users in your database.

![Seed, accounts and Addresses](/img/libraries/accounts_addresses.svg)
