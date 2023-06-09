---
description: The wallet library is a stateful package with a standardized interface for developers to build applications involving IOTA value transactions.
image: /img/logo/logo_dark.svg
keywords:
- layered overview
- high level
- low level
- stronghold
- value transactions
- explanation
---

# Overview

The wallet library is a stateful package with a standardized interface for developers to build applications involving IOTA value transactions. It provides abstractions to handle IOTA payments and can optionally interact with the [IOTA Stronghold enclave](https://github.com/iotaledger/stronghold.rs/) for seed handling, seed storage, and state backup. 

You can read about our specs in detail in our [Engineering Specs Doc](https://github.com/iotaledger/wallet.rs/blob/dev/specs/wallet-ENGINEERING-SPEC-0000.md).

## High Level Layered Overview

![High Level Layered Overview](/img/overview/iota_layers_overview.svg)