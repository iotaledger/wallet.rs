# Overview

The wallet library is a stateful package with a standardized interface for developers to build applications involving IOTA value transactions. It provides abstractions to handle IOTA payments and can optionally interact with [IOTA Stronghold enclave](https://github.com/iotaledger/stronghold.rs/) for seed handling, seed storage and state backup. 

See the full specification [here](https://github.com/iotaledger/wallet.rs/blob/master/specs/wallet-ENGINEERING-SPEC-0000.md).

## High level layered overview:
![iota layers overview](iota_layers_overview.svg)