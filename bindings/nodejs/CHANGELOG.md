# Changelog

## [0.0.3]

-   The balance change event now emits a `{ spent, received }` object with the changed amount instead of the new address balance.
    -   [e5b7b5c](https://github.com/iotaledger/wallet.rs/commit/e5b7b5c85edf118339e4177323da9936ed644558) refactor: balance change event with balance diff instead of new value ([#332](https://github.com/iotaledger/wallet.rs/pull/332)) on 2021-02-18
-   Fixes a panic on the MQTT handling.
    -   [977a71e](https://github.com/iotaledger/wallet.rs/commit/977a71e24e338c8fa1110392b7dcdc83663ec839) fix(mqtt): spawn instead of block_on ([#330](https://github.com/iotaledger/wallet.rs/pull/330)) on 2021-02-17
-   Adds `getUnusedAddress` API on the `Account` class.
    -   [df2f796](https://github.com/iotaledger/wallet.rs/commit/df2f7968b22ef749f7caa177980a8954b44e87ce) feat(nodejs): add getUnusedAddress API ([#327](https://github.com/iotaledger/wallet.rs/pull/327)) on 2021-02-17
-   Fixes issues with the installation script when using with NPM instead of Yarn
    -   [74b10bb](https://github.com/iotaledger/wallet.rs/commit/74b10bbc56d393f1ea650117ba510027a1ae1c0c) fix(bindings/nodejs): Run scripts with NPM instead of Yarn ([#333](https://github.com/iotaledger/wallet.rs/pull/333)) on 2021-02-18
-   Fixes a deadlodk on the account synchronization.
    -   [774b408](https://github.com/iotaledger/wallet.rs/commit/774b4087312c9f8bf6522fb3dfd3e9cb032b88b5) refactor: transaction Essence is now a enum ([#321](https://github.com/iotaledger/wallet.rs/pull/321)) on 2021-02-16
-   Fixes the default initial address index on the account synchronization.
    -   [774b408](https://github.com/iotaledger/wallet.rs/commit/774b4087312c9f8bf6522fb3dfd3e9cb032b88b5) refactor: transaction Essence is now a enum ([#321](https://github.com/iotaledger/wallet.rs/pull/321)) on 2021-02-16
-   The transaction indexation now accepts byte arrays.
    -   [066d515](https://github.com/iotaledger/wallet.rs/commit/066d5155b0e23896b399fc34ca03786836c16278) refactor: transaction index is now a byte array ([#334](https://github.com/iotaledger/wallet.rs/pull/334)) on 2021-02-19

## [0.0.2]

-   Initial release.
    -   [3eb114d](https://github.com/iotaledger/wallet.rs/commit/3eb114d2b3a0bb3956af74aae087ca06724fa7b2) feature(ci, bindings): Node.js bindings publishing and prebuild workflow ([#274](https://github.com/iotaledger/wallet.rs/pull/274)) on 2021-02-11
