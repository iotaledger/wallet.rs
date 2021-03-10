# Changelog

## [0.0.6]

-   The default account alias now starts at index 1.
    -   [c5dad35](https://github.com/iotaledger/wallet.rs/commit/c5dad35f6ec99ba585db035297566c267b24d50b) refactor(account): default alias starts at 1 ([#401](https://github.com/iotaledger/wallet.rs/pull/401)) on 2021-03-09
-   Fixes event storage loading.
    -   [c178419](https://github.com/iotaledger/wallet.rs/commit/c17841928e31b07a0e2172c4ed08d3ede505ede3) fix(storage): load events ([#398](https://github.com/iotaledger/wallet.rs/pull/398)) on 2021-03-08
-   The event persistence is now optional and must be enabled on the AccountManager constructor options.
    -   [8e7461b](https://github.com/iotaledger/wallet.rs/commit/8e7461b2537dff44e4539546d92c5f746486654b) refactor: optional event persistence ([#399](https://github.com/iotaledger/wallet.rs/pull/399)) on 2021-03-09
-   Fixes backup import when using the SQLite database.
    -   [5443848](https://github.com/iotaledger/wallet.rs/commit/544384863771c166278beceb82f70e4ea4f67a3d) fix(manager): save accounts imported from stronghold file ([#396](https://github.com/iotaledger/wallet.rs/pull/396)) on 2021-03-08
-   Fixes an issue with the stronghold status when loading the snapshot with a wrong password.
    -   [5e81e1f](https://github.com/iotaledger/wallet.rs/commit/5e81e1f8e68d87fe50ef89c5a44567299d3de1cf) fix(stronghold): unset password if snapshot loading failed ([#392](https://github.com/iotaledger/wallet.rs/pull/392)) on 2021-03-08
-   Fixes an issue with the account creation when checking if the latest account is empty.
    -   [8d4187f](https://github.com/iotaledger/wallet.rs/commit/8d4187fa00d8f1e941ccaadc5cff41673fcc3735) fix(account): latest account empty check, fixes [#364](https://github.com/iotaledger/wallet.rs/pull/364) ([#394](https://github.com/iotaledger/wallet.rs/pull/394)) on 2021-03-08
-   Updated dependency `rand` to `^0.8` fixing a [security issue](https://github.com/iotaledger/wallet.rs/issues/359).
    -   [44ac325](https://github.com/iotaledger/wallet.rs/commit/44ac325597759c65d9624e8532d2089b4b546564) chore: update dependencies, closes [#359](https://github.com/iotaledger/wallet.rs/pull/359) ([#402](https://github.com/iotaledger/wallet.rs/pull/402)) on 2021-03-09

## [0.0.5]

-   Added auth `username` and `password` to the client options.
    -   [7f462fd](https://github.com/iotaledger/wallet.rs/commit/7f462fd449b490d4761178fb8cc526a865133746) feat(client): add auth options ([#373](https://github.com/iotaledger/wallet.rs/pull/373)) on 2021-03-04
-   Adds a `indexationId` (unique identifier) field to all event payload objects.
    -   [503e2bc](https://github.com/iotaledger/wallet.rs/commit/503e2bcf69d9d3ae5596017f2d7fac20204b3302) refactor(event): add indexation id ([#377](https://github.com/iotaledger/wallet.rs/pull/377)) on 2021-03-05
-   The events are now persisted and the AccountManager has APIs to read them.
    -   [45c9bd9](https://github.com/iotaledger/wallet.rs/commit/45c9bd98192d06b43bcd76c79a16d3324f49fbc2) feat: events persistence ([#356](https://github.com/iotaledger/wallet.rs/pull/356)) on 2021-03-01
-   Fixes the account syncing through the background polling system.
    -   [4fd5068](https://github.com/iotaledger/wallet.rs/commit/4fd5068b7032c57418749e8770f7266cdebf1127) fix(sync): sync on polling should search all addresses, closes [#355](https://github.com/iotaledger/wallet.rs/pull/355) ([#358](https://github.com/iotaledger/wallet.rs/pull/358)) on 2021-03-02
-   Adds the message type filter on the `messageCount` API.
    -   [2fc4e71](https://github.com/iotaledger/wallet.rs/commit/2fc4e7143695fa89c15bbbe9aede7800c4cde5c6) feat(bindings): add filter on message_count API ([#347](https://github.com/iotaledger/wallet.rs/pull/347)) on 2021-02-24
-   Adds a `metadata` field on the transaction essence inputs.
    -   [fd5ae9d](https://github.com/iotaledger/wallet.rs/commit/fd5ae9d7c9337cde0ac6d9edad324f4260296110) refactor(message): add input details on transaction essence inputs ([#361](https://github.com/iotaledger/wallet.rs/pull/361)) on 2021-03-02
-   Addresses on the `Message` object are now serialized with the bech32 format.
    -   [53f90da](https://github.com/iotaledger/wallet.rs/commit/53f90da6610a3ab1762f59b05ae5014acf531174) refactor(message): use Payload wrapper, serde Address as bech32 ([#343](https://github.com/iotaledger/wallet.rs/pull/343)) on 2021-02-23
-   Adds a `remainder` property to the transaction's `output` object.
    -   [f87a987](https://github.com/iotaledger/wallet.rs/commit/f87a9877041fde5dbffed0f117b075450f9ce21f) feat(message): add `remainder` field to the TransactionOutput struct ([#350](https://github.com/iotaledger/wallet.rs/pull/350)) on 2021-02-25
-   Fixes `setStrongholdPassword` accepting a wrong password after a few tries.
    -   [991c2e6](https://github.com/iotaledger/wallet.rs/commit/991c2e68c1f88f0c327d1cd37a1275089aaf0ed3) fix(stronghold): mark client as loaded if the snapshot decrypt succeded ([#357](https://github.com/iotaledger/wallet.rs/pull/357)) on 2021-03-01
-   Adds the `options: SyncOptions` parameter on the `AccountManager#syncAccounts` method.
    -   [9855cfa](https://github.com/iotaledger/wallet.rs/commit/9855cfa4ce7296d04d1c647c7f6ca1722784eb33) refactor(manager): `sync_accounts` gap_limit and address_index options ([#346](https://github.com/iotaledger/wallet.rs/pull/346)) on 2021-02-24
-   Move `transfer`, `retry`, `reattach`, `promote` APIs to the account object.
    -   [8b808c8](https://github.com/iotaledger/wallet.rs/commit/8b808c80bbb7bc1e6b9858551880684a0400ab0c) refactor(sync): automatic sync before transfer/retry/reattach/promote ([#365](https://github.com/iotaledger/wallet.rs/pull/365)) on 2021-03-02
-   Added a `TransferProgress` event type, triggered on transfer steps progress.
    -   [4c46aa6](https://github.com/iotaledger/wallet.rs/commit/4c46aa64ebf6168ca83360ca3df2fcd808103795) feat(transfer): add progress event ([#369](https://github.com/iotaledger/wallet.rs/pull/369)) on 2021-03-02

## [0.0.4]

-   Fixes the message confirmation state update on the background sync system.
    -   [a164f4d](https://github.com/iotaledger/wallet.rs/commit/a164f4d2c844f701744c129aaafb731703a2910f) fix(sync): do not set tx as unconfirmed when the inclusion state is null ([#338](https://github.com/iotaledger/wallet.rs/pull/338)) on 2021-02-19
-   New method on the Account object to get an address by its bech32 representation.
    -   [0652cd9](https://github.com/iotaledger/wallet.rs/commit/0652cd93c620323026720e43c0510791901ba35c) feat(nodejs): add API to get an address by bech32 string ([#336](https://github.com/iotaledger/wallet.rs/pull/336)) on 2021-02-19
-   Adds a `messageCount` function on the Account class.
    -   [ed74aaf](https://github.com/iotaledger/wallet.rs/commit/ed74aaf3a8ffe6737b17e86455326811c9e52f76) feat(bindings): add messageCount API ([#340](https://github.com/iotaledger/wallet.rs/pull/340)) on 2021-02-22

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
