# Changelog

## \[0.3.7]

- Fixes updating client options.
  - [8124c3de](https://github.com/iotaledger/wallet.rs/commit/8124c3de93f006c20a1e0640d89fbfb6ab42d325) use iota_client ([#611](https://github.com/iotaledger/wallet.rs/pull/611)) on 2021-05-10

## \[0.3.6]

- Fix rev in `native/Cargo.toml`
  - [c2caecaa](https://github.com/iotaledger/wallet.rs/commit/c2caecaaa69ad5fd9a98c346f3c3a599573679cc) fix(ci): Fix covector config on 2021-05-05

## \[0.3.5]

- Set git repo and rev to allow JS bindings to be built from source
  - [ab7556e8](https://github.com/iotaledger/wallet.rs/commit/ab7556e88322e89fb37876186a5bca1329c752fc) fix(bindings/nodejs): Set rev to allow building from source ([#601](https://github.com/iotaledger/wallet.rs/pull/601)) on 2021-05-05

## \[0.3.4]

- Fix `rocksdb` publish issue on v0.3.3
  - [203ede63](https://github.com/iotaledger/wallet.rs/commit/203ede633c3f22c1773240beef3c2100b5953bbd) fix: publish ([#594](https://github.com/iotaledger/wallet.rs/pull/594)) on 2021-04-27

## \[0.3.3]

- Fixes `account.sync` generating a change address on each call.
  - [8afe3deb](https://github.com/iotaledger/wallet.rs/commit/8afe3deb021fc2b31be6045ed3d4c1d35de149ec) fix(sync): initial address index should be the max on internal addresses ([#592](https://github.com/iotaledger/wallet.rs/pull/592)) on 2021-04-27
- Adds `MigrationProgress` event type.
  - [5b72899c](https://github.com/iotaledger/wallet.rs/commit/5b72899c942b99d67ddfa3bd2fb6a0261c646e0c) feat(bindings): implement migration progress events ([#591](https://github.com/iotaledger/wallet.rs/pull/591)) on 2021-04-27

## \[0.3.2]

- Drop `.stronghold` extension requirement on wallet backup.
  - [0e849b30](https://github.com/iotaledger/wallet.rs/commit/0e849b3048ce4dc6639b9eddf35ac8d878f20fe2) chore(manager): simplify backup API, dropping extension requirement ([#588](https://github.com/iotaledger/wallet.rs/pull/588)) on 2021-04-26

## \[0.3.1]

- Prevent `storage adapter not set` errors.
  - [af8c4195](https://github.com/iotaledger/wallet.rs/commit/af8c419525c97164578187cd748d622616ff9f6c) fix(manager): prevent `storage adapter not set` errors ([#584](https://github.com/iotaledger/wallet.rs/pull/584)) on 2021-04-22

## \[0.3.0]

- Updates Stronghold to latest refactor (breaking change).
  - [2a1cb6e3](https://github.com/iotaledger/wallet.rs/commit/2a1cb6e3a7a578e5cc93f45b439ce680b54d31ac) chore(deps): update to latest stronghold.rs ([#577](https://github.com/iotaledger/wallet.rs/pull/577)) on 2021-04-19

## \[0.2.4]

- Fixes `account.balance()` return value.
  - [a439109c](https://github.com/iotaledger/wallet.rs/commit/a439109c4008f33dddbf9ea7e41f39d90e39a8f4) fix(nodejs): `account.balance` return value ([#572](https://github.com/iotaledger/wallet.rs/pull/572)) on 2021-04-19

## \[0.2.3]

- Adds an option to enable creating multiple accounts without history.
  - [5e9e2c92](https://github.com/iotaledger/wallet.rs/commit/5e9e2c92999f0261442f2b875ff8483d631088c9) feat(manager): add option to allow creating multiple empty accounts ([#567](https://github.com/iotaledger/wallet.rs/pull/567)) on 2021-04-19
- Implement `sendToMany` API on the `Account` object.
  - [73656767](https://github.com/iotaledger/wallet.rs/commit/73656767f25f7e5ba3097f83a5fc788024c87c9c) Added multi output transfer API ([#557](https://github.com/iotaledger/wallet.rs/pull/557)) on 2021-04-17

## \[0.2.2]

- Updates Testnet breaking changes and includes several fixes to syncing and change address management.
  - [90ca9368](https://github.com/iotaledger/wallet.rs/commit/90ca9368c28a04ed9289bb10bc05d3800bc1a47e) chore: add change file ([#546](https://github.com/iotaledger/wallet.rs/pull/546)) on 2021-04-13

## \[0.2.1]

- Fix Windows CI
  - [df4e38f5](https://github.com/iotaledger/wallet.rs/commit/df4e38f539af44b24f2372d154b4533d9c5f80f1) fix(ci): Install LLVM and Clang on Windows ([#516](https://github.com/iotaledger/wallet.rs/pull/516)) on 2021-04-08
- Reuse RocksDB storage instances.
  - [d425e99f](https://github.com/iotaledger/wallet.rs/commit/d425e99fc7501656f6816f6cf8e03f1c8353104b) fix(manager): reuse existing rocksdb instances ([#524](https://github.com/iotaledger/wallet.rs/pull/524)) on 2021-04-09

## \[0.2.0]

- Refactor `Message` object storage for performance.
  - [56fad662](https://github.com/iotaledger/wallet.rs/commit/56fad66285932d26052f147f1599ec2664fabb93) refactor(storage): separate Message list on db, closes [#480](https://github.com/iotaledger/wallet.rs/pull/480) ([#493](https://github.com/iotaledger/wallet.rs/pull/493)) on 2021-04-06
- Use RocksDB instead of SQLite as database.
  - [66831376](https://github.com/iotaledger/wallet.rs/commit/66831376b124e574829d0566f79879af8b23dde2) refactor(storage): use RocksDB instead of SQLite ([#471](https://github.com/iotaledger/wallet.rs/pull/471)) on 2021-04-06

## \[0.1.1]

- Adds Node v15 support.
  - [5fd13b4](https://github.com/iotaledger/wallet.rs/commit/5fd13b43e0af8ce59a671238e00ef42647cb28fd) feat(ci): prebuild for node v15 ([#494](https://github.com/iotaledger/wallet.rs/pull/494)) on 2021-04-02

## \[0.1.0]

- Adds a `password` field on the `backup` API.
  - [5c428c6](https://github.com/iotaledger/wallet.rs/commit/5c428c639c7ff3580eb3cc0c8852ac3ab53bf2b8) refactor: remove custom storage option ([#466](https://github.com/iotaledger/wallet.rs/pull/466)) on 2021-03-26
- Backup destination can now be a path to a file instead of a directory, allowing custom filenames.
  - [ffbeaa3](https://github.com/iotaledger/wallet.rs/commit/ffbeaa3466b44f79dd5f87e14ed1bdc4846d9e85) feat(backup): allow file path as destination for custom filenames ([#426](https://github.com/iotaledger/wallet.rs/pull/426)) on 2021-03-14
- Adds a `messageId` field to the balance change event payload.
  - [c7d34e2](https://github.com/iotaledger/wallet.rs/commit/c7d34e213a7a42503b21714847c9642e19878cb4) feat(events): reintroduce message_ids on balance change event payload ([#406](https://github.com/iotaledger/wallet.rs/pull/406)) on 2021-03-09
  - [1e4447c](https://github.com/iotaledger/wallet.rs/commit/1e4447cf161940d17707e14f544c105e88ddff54) refactor(event): split balance change events on message id ([#412](https://github.com/iotaledger/wallet.rs/pull/412)) on 2021-03-11
- The `incoming` and `outgoing` account balances now ignores internal transactions.
  - [f1dbd05](https://github.com/iotaledger/wallet.rs/commit/f1dbd05b4347ed649cf76458e21d6c8bf1cf68c4) refactor(message): detect internal txs, move value fields, fix balance ([#407](https://github.com/iotaledger/wallet.rs/pull/407)) on 2021-03-09
- Properly validate the `currentPassword` on the `changeStrongholdPassword` API.
  - [ce685aa](https://github.com/iotaledger/wallet.rs/commit/ce685aadb8c76d61bb13f9c46c35526a22f25e89) fix(stronghold): properly check current password on password change API ([#408](https://github.com/iotaledger/wallet.rs/pull/408)) on 2021-03-10
- Fixes duplicated balance change and new transaction events being triggered.
  - [8bbca83](https://github.com/iotaledger/wallet.rs/commit/8bbca83a0a9c23025823def680a9a123d63561f6) fix(sync): lock the account so duplicated events never happen ([#403](https://github.com/iotaledger/wallet.rs/pull/403)) on 2021-03-09
- Moved message fields `value`, `incoming`, `remainderValue` to the `RegularEssence` object.
  - [f1dbd05](https://github.com/iotaledger/wallet.rs/commit/f1dbd05b4347ed649cf76458e21d6c8bf1cf68c4) refactor(message): detect internal txs, move value fields, fix balance ([#407](https://github.com/iotaledger/wallet.rs/pull/407)) on 2021-03-09
- Adds `disabled` flag on the `Node` object.
  - [782ebfd](https://github.com/iotaledger/wallet.rs/commit/782ebfd458fe5e7ff070b3055c708e18000fb607) feat(client): add `disabled` option to the node struct ([#484](https://github.com/iotaledger/wallet.rs/pull/484)) on 2021-03-30
- Adds `reattachedMessageId` field on the reattachment event payload.
  - [2f2ccee](https://github.com/iotaledger/wallet.rs/commit/2f2ccee3d2799ae40219ee52fdc1c364e45cef3c) feat(events): add `reattached_message_id` on reattachment event ([#432](https://github.com/iotaledger/wallet.rs/pull/432)) on 2021-03-16
- Removes the `StorageType` option.
  - [5c428c6](https://github.com/iotaledger/wallet.rs/commit/5c428c639c7ff3580eb3cc0c8852ac3ab53bf2b8) refactor: remove custom storage option ([#466](https://github.com/iotaledger/wallet.rs/pull/466)) on 2021-03-26
- Prevent overwriting the Stronghold mnemonic by throwing an error.
  - [eaf3763](https://github.com/iotaledger/wallet.rs/commit/eaf3763215c0f58513bfac0408ec8a573123e71d) feat(stronghold): check if mnemonic is already set, closes [#409](https://github.com/iotaledger/wallet.rs/pull/409) ([#486](https://github.com/iotaledger/wallet.rs/pull/486)) on 2021-03-31
- Fixes address outputs syncing.
  - [67fd04f](https://github.com/iotaledger/wallet.rs/commit/67fd04fc7e27a9a6e33eb1851df6cbc29dd77022) fix(sync): fetch output from the node if local copy is unspent ([#454](https://github.com/iotaledger/wallet.rs/pull/454)) on 2021-03-21
- The wallet now validates the nodes provided to the account creation and the `setClientOptions` API.
  - [a77fb60](https://github.com/iotaledger/wallet.rs/commit/a77fb60a26e8df5de79c5b3accc5412d93061af7) feat(account): add client options validation ([#404](https://github.com/iotaledger/wallet.rs/pull/404)) on 2021-03-09

## \[0.0.6]

- The default account alias now starts at index 1.
  - [c5dad35](https://github.com/iotaledger/wallet.rs/commit/c5dad35f6ec99ba585db035297566c267b24d50b) refactor(account): default alias starts at 1 ([#401](https://github.com/iotaledger/wallet.rs/pull/401)) on 2021-03-09
- Fixes event storage loading.
  - [c178419](https://github.com/iotaledger/wallet.rs/commit/c17841928e31b07a0e2172c4ed08d3ede505ede3) fix(storage): load events ([#398](https://github.com/iotaledger/wallet.rs/pull/398)) on 2021-03-08
- The event persistence is now optional and must be enabled on the AccountManager constructor options.
  - [8e7461b](https://github.com/iotaledger/wallet.rs/commit/8e7461b2537dff44e4539546d92c5f746486654b) refactor: optional event persistence ([#399](https://github.com/iotaledger/wallet.rs/pull/399)) on 2021-03-09
- Fixes backup import when using the SQLite database.
  - [5443848](https://github.com/iotaledger/wallet.rs/commit/544384863771c166278beceb82f70e4ea4f67a3d) fix(manager): save accounts imported from stronghold file ([#396](https://github.com/iotaledger/wallet.rs/pull/396)) on 2021-03-08
- Fixes an issue with the stronghold status when loading the snapshot with a wrong password.
  - [5e81e1f](https://github.com/iotaledger/wallet.rs/commit/5e81e1f8e68d87fe50ef89c5a44567299d3de1cf) fix(stronghold): unset password if snapshot loading failed ([#392](https://github.com/iotaledger/wallet.rs/pull/392)) on 2021-03-08
- Fixes an issue with the account creation when checking if the latest account is empty.
  - [8d4187f](https://github.com/iotaledger/wallet.rs/commit/8d4187fa00d8f1e941ccaadc5cff41673fcc3735) fix(account): latest account empty check, fixes [#364](https://github.com/iotaledger/wallet.rs/pull/364) ([#394](https://github.com/iotaledger/wallet.rs/pull/394)) on 2021-03-08
- Updated dependency `rand` to `^0.8` fixing a [security issue](https://github.com/iotaledger/wallet.rs/issues/359).
  - [44ac325](https://github.com/iotaledger/wallet.rs/commit/44ac325597759c65d9624e8532d2089b4b546564) chore: update dependencies, closes [#359](https://github.com/iotaledger/wallet.rs/pull/359) ([#402](https://github.com/iotaledger/wallet.rs/pull/402)) on 2021-03-09

## \[0.0.5]

- Added auth `username` and `password` to the client options.
  - [7f462fd](https://github.com/iotaledger/wallet.rs/commit/7f462fd449b490d4761178fb8cc526a865133746) feat(client): add auth options ([#373](https://github.com/iotaledger/wallet.rs/pull/373)) on 2021-03-04
- Adds a `indexationId` (unique identifier) field to all event payload objects.
  - [503e2bc](https://github.com/iotaledger/wallet.rs/commit/503e2bcf69d9d3ae5596017f2d7fac20204b3302) refactor(event): add indexation id ([#377](https://github.com/iotaledger/wallet.rs/pull/377)) on 2021-03-05
- The events are now persisted and the AccountManager has APIs to read them.
  - [45c9bd9](https://github.com/iotaledger/wallet.rs/commit/45c9bd98192d06b43bcd76c79a16d3324f49fbc2) feat: events persistence ([#356](https://github.com/iotaledger/wallet.rs/pull/356)) on 2021-03-01
- Fixes the account syncing through the background polling system.
  - [4fd5068](https://github.com/iotaledger/wallet.rs/commit/4fd5068b7032c57418749e8770f7266cdebf1127) fix(sync): sync on polling should search all addresses, closes [#355](https://github.com/iotaledger/wallet.rs/pull/355) ([#358](https://github.com/iotaledger/wallet.rs/pull/358)) on 2021-03-02
- Adds the message type filter on the `messageCount` API.
  - [2fc4e71](https://github.com/iotaledger/wallet.rs/commit/2fc4e7143695fa89c15bbbe9aede7800c4cde5c6) feat(bindings): add filter on message_count API ([#347](https://github.com/iotaledger/wallet.rs/pull/347)) on 2021-02-24
- Adds a `metadata` field on the transaction essence inputs.
  - [fd5ae9d](https://github.com/iotaledger/wallet.rs/commit/fd5ae9d7c9337cde0ac6d9edad324f4260296110) refactor(message): add input details on transaction essence inputs ([#361](https://github.com/iotaledger/wallet.rs/pull/361)) on 2021-03-02
- Addresses on the `Message` object are now serialized with the bech32 format.
  - [53f90da](https://github.com/iotaledger/wallet.rs/commit/53f90da6610a3ab1762f59b05ae5014acf531174) refactor(message): use Payload wrapper, serde Address as bech32 ([#343](https://github.com/iotaledger/wallet.rs/pull/343)) on 2021-02-23
- Adds a `remainder` property to the transaction's `output` object.
  - [f87a987](https://github.com/iotaledger/wallet.rs/commit/f87a9877041fde5dbffed0f117b075450f9ce21f) feat(message): add `remainder` field to the TransactionOutput struct ([#350](https://github.com/iotaledger/wallet.rs/pull/350)) on 2021-02-25
- Fixes `setStrongholdPassword` accepting a wrong password after a few tries.
  - [991c2e6](https://github.com/iotaledger/wallet.rs/commit/991c2e68c1f88f0c327d1cd37a1275089aaf0ed3) fix(stronghold): mark client as loaded if the snapshot decrypt succeded ([#357](https://github.com/iotaledger/wallet.rs/pull/357)) on 2021-03-01
- Adds the `options: SyncOptions` parameter on the `AccountManager#syncAccounts` method.
  - [9855cfa](https://github.com/iotaledger/wallet.rs/commit/9855cfa4ce7296d04d1c647c7f6ca1722784eb33) refactor(manager): `sync_accounts` gap_limit and address_index options ([#346](https://github.com/iotaledger/wallet.rs/pull/346)) on 2021-02-24
- Move `transfer`, `retry`, `reattach`, `promote` APIs to the account object.
  - [8b808c8](https://github.com/iotaledger/wallet.rs/commit/8b808c80bbb7bc1e6b9858551880684a0400ab0c) refactor(sync): automatic sync before transfer/retry/reattach/promote ([#365](https://github.com/iotaledger/wallet.rs/pull/365)) on 2021-03-02
- Added a `TransferProgress` event type, triggered on transfer steps progress.
  - [4c46aa6](https://github.com/iotaledger/wallet.rs/commit/4c46aa64ebf6168ca83360ca3df2fcd808103795) feat(transfer): add progress event ([#369](https://github.com/iotaledger/wallet.rs/pull/369)) on 2021-03-02

## \[0.0.4]

- Fixes the message confirmation state update on the background sync system.
  - [a164f4d](https://github.com/iotaledger/wallet.rs/commit/a164f4d2c844f701744c129aaafb731703a2910f) fix(sync): do not set tx as unconfirmed when the inclusion state is null ([#338](https://github.com/iotaledger/wallet.rs/pull/338)) on 2021-02-19
- New method on the Account object to get an address by its bech32 representation.
  - [0652cd9](https://github.com/iotaledger/wallet.rs/commit/0652cd93c620323026720e43c0510791901ba35c) feat(nodejs): add API to get an address by bech32 string ([#336](https://github.com/iotaledger/wallet.rs/pull/336)) on 2021-02-19
- Adds a `messageCount` function on the Account class.
  - [ed74aaf](https://github.com/iotaledger/wallet.rs/commit/ed74aaf3a8ffe6737b17e86455326811c9e52f76) feat(bindings): add messageCount API ([#340](https://github.com/iotaledger/wallet.rs/pull/340)) on 2021-02-22

## \[0.0.3]

- The balance change event now emits a `{ spent, received }` object with the changed amount instead of the new address balance.
  - [e5b7b5c](https://github.com/iotaledger/wallet.rs/commit/e5b7b5c85edf118339e4177323da9936ed644558) refactor: balance change event with balance diff instead of new value ([#332](https://github.com/iotaledger/wallet.rs/pull/332)) on 2021-02-18
- Fixes a panic on the MQTT handling.
  - [977a71e](https://github.com/iotaledger/wallet.rs/commit/977a71e24e338c8fa1110392b7dcdc83663ec839) fix(mqtt): spawn instead of block_on ([#330](https://github.com/iotaledger/wallet.rs/pull/330)) on 2021-02-17
- Adds `getUnusedAddress` API on the `Account` class.
  - [df2f796](https://github.com/iotaledger/wallet.rs/commit/df2f7968b22ef749f7caa177980a8954b44e87ce) feat(nodejs): add getUnusedAddress API ([#327](https://github.com/iotaledger/wallet.rs/pull/327)) on 2021-02-17
- Fixes issues with the installation script when using with NPM instead of Yarn
  - [74b10bb](https://github.com/iotaledger/wallet.rs/commit/74b10bbc56d393f1ea650117ba510027a1ae1c0c) fix(bindings/nodejs): Run scripts with NPM instead of Yarn ([#333](https://github.com/iotaledger/wallet.rs/pull/333)) on 2021-02-18
- Fixes a deadlodk on the account synchronization.
  - [774b408](https://github.com/iotaledger/wallet.rs/commit/774b4087312c9f8bf6522fb3dfd3e9cb032b88b5) refactor: transaction Essence is now a enum ([#321](https://github.com/iotaledger/wallet.rs/pull/321)) on 2021-02-16
- Fixes the default initial address index on the account synchronization.
  - [774b408](https://github.com/iotaledger/wallet.rs/commit/774b4087312c9f8bf6522fb3dfd3e9cb032b88b5) refactor: transaction Essence is now a enum ([#321](https://github.com/iotaledger/wallet.rs/pull/321)) on 2021-02-16
- The transaction indexation now accepts byte arrays.
  - [066d515](https://github.com/iotaledger/wallet.rs/commit/066d5155b0e23896b399fc34ca03786836c16278) refactor: transaction index is now a byte array ([#334](https://github.com/iotaledger/wallet.rs/pull/334)) on 2021-02-19

## \[0.0.2]

- Initial release.
  - [3eb114d](https://github.com/iotaledger/wallet.rs/commit/3eb114d2b3a0bb3956af74aae087ca06724fa7b2) feature(ci, bindings): Node.js bindings publishing and prebuild workflow ([#274](https://github.com/iotaledger/wallet.rs/pull/274)) on 2021-02-11
