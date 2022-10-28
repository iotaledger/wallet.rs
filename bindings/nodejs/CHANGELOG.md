# Changelog

## \[2.0.3-rc.2]

- Fixed NFT output claiming.
  - [60d55da0](https://github.com/iotaledger/wallet.rs/commit/60d55da011ae0f8b6c0288aa8a96411d20b2b191) Fix NFT output claiming ([#1504](https://github.com/iotaledger/wallet.rs/pull/1504)) on 2022-10-19
- Alias and NFT owned outputs syncing;
  Alias or nft addresses in address unlock condition;
  - [0a80a29e](https://github.com/iotaledger/wallet.rs/commit/0a80a29e497cd13932033a208e987fea15de3bbf) Fix alias/nft outputs owned syncing ([#1479](https://github.com/iotaledger/wallet.rs/pull/1479)) on 2022-10-28
- Add transaction essence and payload length validation.
  - [3949b95c](https://github.com/iotaledger/wallet.rs/commit/3949b95ce2740cdcbe9775a542336a0261d601b7) Tx length validation ([#1500](https://github.com/iotaledger/wallet.rs/pull/1500)) on 2022-10-21

## \[2.0.3-rc.1]

- Add getAccountIndexes().
  - [6e20b261](https://github.com/iotaledger/wallet.rs/commit/6e20b2611132e3a92f867b2d80333324485666e2) Add GetAccountIndexes, fix create account example ([#1480](https://github.com/iotaledger/wallet.rs/pull/1480)) on 2022-09-30
- Fix incomingTransactions() return type definition.
  - [ee17b037](https://github.com/iotaledger/wallet.rs/commit/ee17b0377f7060aa3454cfb808fa77b517a32d69) Fix incomingTransactions() return type definition ([#1476](https://github.com/iotaledger/wallet.rs/pull/1476)) on 2022-09-29

## \[2.0.3-rc.0]

- Fix incomingTransactions() return type definition.
  - [450e93cb](https://github.com/iotaledger/wallet.rs/commit/450e93cbef3ed821757757348df9720aebf3510e) Fix incomingTransactions() return type definition ([#1474](https://github.com/iotaledger/wallet.rs/pull/1474)) on 2022-09-28
- Add `sender`, `tag` and `issuer` fields to `NftOptions`.
  - [75c68220](https://github.com/iotaledger/wallet.rs/commit/75c682202c06b7d7d7677e2a77e73e60f2630ac6) Add features missing from NftOptions ([#1412](https://github.com/iotaledger/wallet.rs/pull/1412)) on 2022-09-28
- Make account meta private again.
  - [aa3ed622](https://github.com/iotaledger/wallet.rs/commit/aa3ed6228e649ecc4d3bce30a0fb364b0dfdce8d) Make account meta private again ([#1471](https://github.com/iotaledger/wallet.rs/pull/1471)) on 2022-09-28
- Remove broken deleteAccountsAndDatabase().
  - [cfc19ac6](https://github.com/iotaledger/wallet.rs/commit/cfc19ac69ef40a2cac57642dae6b3369cdf5d8eb) Remove broken delete_accounts_and_database() ([#1455](https://github.com/iotaledger/wallet.rs/pull/1455)) on 2022-09-23
- Remove `list` prefix from functions.
  - [6e1df531](https://github.com/iotaledger/wallet.rs/commit/6e1df5313f27a560aca98cfa17d544abeff08124) Remove "list" prefix from functions ([#1456](https://github.com/iotaledger/wallet.rs/pull/1456)) on 2022-09-23
- Use `Uint8Array` over `number[]` in `IAliasOutputBuilderOptions` and other places to better reflect the type requirements.
  - [e63ace19](https://github.com/iotaledger/wallet.rs/commit/e63ace1969c7b815046eda7ae65ef3bde3ee2449) Use `Uint8Array` over `number[]` ([#1457](https://github.com/iotaledger/wallet.rs/pull/1457)) on 2022-09-23

## \[2.0.2-alpha.32]

- Fix claimOutputs() with additional native tokens in the inputs.
  - [30d17879](https://github.com/iotaledger/wallet.rs/commit/30d178797af26078e94641cd4c185328d41e599d) Fix output claiming with native tokens on 2022-09-28

## \[2.0.2-alpha.31]

- Attempt to fix deadlock in recoverAccounts()
  - [a2eb9e9a](https://github.com/iotaledger/wallet.rs/commit/a2eb9e9af9e736f173d946cd76ccc4e11f638caa) Attempt to fix deadlock on 2022-09-27

## \[2.0.2-alpha.30]

- Fix address start indexes in recoverAccounts().
  - [801f426e](https://github.com/iotaledger/wallet.rs/commit/801f426ebd7a2ea8528bc3a220f4391151286aa9) Fix address start indexes in recoverAccounts() on 2022-09-27

## \[2.0.2-alpha.29]

- Fix sync options in recoverAccounts().
  - [8a382b45](https://github.com/iotaledger/wallet.rs/commit/8a382b45d3a2bea92ab1cdf17eaa06e6a5cbffc4) Fix sync options in recoverAccounts() on 2022-09-27

## \[2.0.2-alpha.28]

- Add createAliasOutput().
  Don't automatically create an alias output in mintNativeToken().
  - [8207a475](https://github.com/iotaledger/wallet.rs/commit/8207a4758b1c0addd744872f893f14d5c2066aa4) Add create_alias_output ([#1438](https://github.com/iotaledger/wallet.rs/pull/1438)) on 2022-09-22
- Fix wrong spent output status.
  - [f8a31305](https://github.com/iotaledger/wallet.rs/commit/f8a31305fabef6256b4559ecc31c2c2242f4448e) Fix spent outputs ([#1449](https://github.com/iotaledger/wallet.rs/pull/1449)) on 2022-09-22
- Removed automatic syncing after tx confirmation.
  - [f8a31305](https://github.com/iotaledger/wallet.rs/commit/f8a31305fabef6256b4559ecc31c2c2242f4448e) Fix spent outputs ([#1449](https://github.com/iotaledger/wallet.rs/pull/1449)) on 2022-09-22

## \[2.0.2-alpha.27]

- Update `@iota/types` dependency to fix a compilation issue.
  - [b6b09ae1](https://github.com/iotaledger/wallet.rs/commit/b6b09ae10afa894da07c153c0a6416a7e9cfe22f) Update `@iota/types` dependency to fix a compilation issue ([#1447](https://github.com/iotaledger/wallet.rs/pull/1447)) on 2022-09-21

## \[2.0.2-alpha.26]

- Make account meta field temporarily public.
  - [5174b782](https://github.com/iotaledger/wallet.rs/commit/5174b782bdd03bdabd2a6aa59f9c5e752a9d7aa5) Make account meta field temporarily public ([#1444](https://github.com/iotaledger/wallet.rs/pull/1444)) on 2022-09-21

## \[2.0.2-alpha.25]

- Remove tryClaimOutputs() and only do a single transaction when calling the claimOutputs() function.
  - [ffb5f867](https://github.com/iotaledger/wallet.rs/commit/ffb5f8677feca3493d1c751490da885e3eefb626) Remove try_claim_outputs() ([#1434](https://github.com/iotaledger/wallet.rs/pull/1434)) on 2022-09-15
- Make account meta private.
  Replace getAlias() with getMetadata().
  - [0ee85b48](https://github.com/iotaledger/wallet.rs/commit/0ee85b48a8ebada02d39a23e1f98d1d7457a6d31) Make account meta private ([#1428](https://github.com/iotaledger/wallet.rs/pull/1428)) on 2022-09-20
- Only do a single transaction in consolidation.
  - [a45278f2](https://github.com/iotaledger/wallet.rs/commit/a45278f2c06058a24bd056a9f91f08d3e42fa9fa) Only do a single consolidation transaction ([#1437](https://github.com/iotaledger/wallet.rs/pull/1437)) on 2022-09-16
- Remove outdated ErrorThrown event.
  - [e39716ad](https://github.com/iotaledger/wallet.rs/commit/e39716ad797b216449763ce33b03ea75e11b3f94) Remove outdated ErorThrown event ([#1443](https://github.com/iotaledger/wallet.rs/pull/1443)) on 2022-09-20
- Fix dependencies.
  - [48378b39](https://github.com/iotaledger/wallet.rs/commit/48378b3927d89a4e71e914c11de9e05aeb65276e) Fix npm dependencies ([#1431](https://github.com/iotaledger/wallet.rs/pull/1431)) on 2022-09-13
- Fix minted tokens amount in increaseNativeTokenSupply().
  - [58df4927](https://github.com/iotaledger/wallet.rs/commit/58df4927d4894b300d929d5099faa654459c82fc) Fix mint amount ([#1435](https://github.com/iotaledger/wallet.rs/pull/1435)) on 2022-09-15
- Add missing `coin_type` on account manager backups.
  - [66d94faf](https://github.com/iotaledger/wallet.rs/commit/66d94faf8b860f3287bed28e207fa7d688890340) Add missing `coin_type` on account manager backups ([#1422](https://github.com/iotaledger/wallet.rs/pull/1422)) on 2022-09-08
- Renamed meltNativeToken() to decreaseNativeTokenSupply() and aligned parameters with burnNativeToken() and increaseNativeTokenSupply().
  Added increaseNativeTokenSupply().
  - [b29be7c0](https://github.com/iotaledger/wallet.rs/commit/b29be7c0b620b679729e58408809418ea5d921c9) Mint more native token ([#1418](https://github.com/iotaledger/wallet.rs/pull/1418)) on 2022-09-13
- Add accountStartIndex to recoverAccounts and change gap limit logic.
  - [4cee3a33](https://github.com/iotaledger/wallet.rs/commit/4cee3a33711d41e08d6fd4a372be42597680ccb1) Update recover accounts ([#1433](https://github.com/iotaledger/wallet.rs/pull/1433)) on 2022-09-20
- Add SeedSecretManager.
  - [44e85926](https://github.com/iotaledger/wallet.rs/commit/44e85926592f854a8e39aa4f63c7e35272ed89cb) Add SeedSecretManager ([#1430](https://github.com/iotaledger/wallet.rs/pull/1430)) on 2022-09-14

## \[2.0.2-alpha.24]

- Added optional FilterOptions to listOutputs() and listUnspentOutputs().
  - [11bddc7f](https://github.com/iotaledger/wallet.rs/commit/11bddc7f12dbd5ed18c7fd9a5582f26e03573f80) Add FilterOptions ([#1413](https://github.com/iotaledger/wallet.rs/pull/1413)) on 2022-09-07
- Check max native tokens limit in consolidation and claiming.
  - [fa895233](https://github.com/iotaledger/wallet.rs/commit/fa8952331a56ed74a1a87786eed8d166c7cf8082) Check max native tokens limit in consolidation and claiming ([#1411](https://github.com/iotaledger/wallet.rs/pull/1411)) on 2022-09-07
- Return correct Account type from recoverAccounts().
  - [909f80ae](https://github.com/iotaledger/wallet.rs/commit/909f80aea217280e134bc298d1fee6e7a7529358) Return Account instead of AccountMeta ([#1409](https://github.com/iotaledger/wallet.rs/pull/1409)) on 2022-09-05

## \[2.0.2-alpha.23]

- Bump client to update `iota-ledger-nano` and fix a bug.
  - [c2463433](https://github.com/iotaledger/wallet.rs/commit/c2463433c92ef33b71db9d0611006a0e8feb447f) Bump client to update `iota-ledger-nano` and fix a bug ([#1408](https://github.com/iotaledger/wallet.rs/pull/1408)) on 2022-09-02
- Bump client revision to include the fix `Check expiration for remainder address`.
  - [eac49410](https://github.com/iotaledger/wallet.rs/commit/eac49410cd5c8b48c229579704e792e93a516a6d) Check expiration for remainder address ([#1406](https://github.com/iotaledger/wallet.rs/pull/1406)) on 2022-09-02
  - [c2463433](https://github.com/iotaledger/wallet.rs/commit/c2463433c92ef33b71db9d0611006a0e8feb447f) Bump client to update `iota-ledger-nano` and fix a bug ([#1408](https://github.com/iotaledger/wallet.rs/pull/1408)) on 2022-09-02

## \[2.0.2-alpha.22]

- Bumped client revision and fix breaking changes.
  - [1bbec445](https://github.com/iotaledger/wallet.rs/commit/1bbec4454462492e7de4b79d40c970e7454613d5) Add .changes for nodejs bindings ([#1395](https://github.com/iotaledger/wallet.rs/pull/1395)) on 2022-08-29

## \[2.0.2-alpha.21]

- Update prebuild-install to 7.1.1 and specify building for electron latest version.
  - [2b5fd77e](https://github.com/iotaledger/wallet.rs/commit/2b5fd77ec3ccb963b82e9d6e89b3012a6520d3e6) chore: update to ubuntu 20.04 ([#1384](https://github.com/iotaledger/wallet.rs/pull/1384)) on 2022-08-22

## \[2.0.2-alpha.20]

- Fix prebuilding the electron bindings
  - [cacd6853](https://github.com/iotaledger/wallet.rs/commit/cacd6853abc367765268ef193aaee39a73d1ca02) fix: electron prebuilds ([#1380](https://github.com/iotaledger/wallet.rs/pull/1380)) on 2022-08-22

## \[2.0.2-alpha.19]

- Fixed location of native add-on.
  - [2a4409cb](https://github.com/iotaledger/wallet.rs/commit/2a4409cb11993f4102c09e97b27929183907dfda) chore: add changes to trigger covector ([#1378](https://github.com/iotaledger/wallet.rs/pull/1378)) on 2022-08-19

## \[2.0.2-alpha.18]

- Striping index.node from the correct path
  - [e9cc4b7c](https://github.com/iotaledger/wallet.rs/commit/e9cc4b7ca7fa377b6c8dd982a826c2552869f9f9) fix: workflow doesn't support matrix exclusions ([#1374](https://github.com/iotaledger/wallet.rs/pull/1374)) on 2022-08-19

## \[2.0.2-alpha.17]

- Fixed prebuilds for nodejs bindings.
  Add newer Electron versions for electron build.
  - [a3381e5a](https://github.com/iotaledger/wallet.rs/commit/a3381e5ad5cdde78291f0765637056b09772a76d) fix: fix nodejs prebuilds ([#1372](https://github.com/iotaledger/wallet.rs/pull/1372)) on 2022-08-19
- Add missing typescript dependency.
  - [7f885322](https://github.com/iotaledger/wallet.rs/commit/7f8853225d9f3afc90d9ec8130a65fdb87fa4539) Add missing typescript dependency ([#1371](https://github.com/iotaledger/wallet.rs/pull/1371)) on 2022-08-19

## \[2.0.2-alpha.16]

- Fix removeLatestAccount().
  - [0d1ad1c0](https://github.com/iotaledger/wallet.rs/commit/0d1ad1c07c4968237909867e25a3263ba1c51cbc) Fix remove_latest_account ([#1369](https://github.com/iotaledger/wallet.rs/pull/1369)) on 2022-08-18
- Emit `PreparedTransaction` only once and when blindsigning is not needed.
  - [d5479e61](https://github.com/iotaledger/wallet.rs/commit/d5479e61b59d2836f7174aeb31d24b2d18ed4a27) Emit `PreparedTransaction` only once and when blindsigning is not needed ([#1367](https://github.com/iotaledger/wallet.rs/pull/1367)) on 2022-08-18

## \[2.0.2-alpha.15]

- Add "win_delay_load_hook": "true" to check if add-on gets compiled correctly.
  - [6dab8026](https://github.com/iotaledger/wallet.rs/commit/6dab80263f7d32284c29b6497d802778441d42b9) fix: windows delay load hook set to true ([#1363](https://github.com/iotaledger/wallet.rs/pull/1363)) on 2022-08-16

## \[2.0.2-alpha.14]

- Fix restore backup.
  - [0e849b30](https://github.com/iotaledger/wallet.rs/commit/0e849b3048ce4dc6639b9eddf35ac8d878f20fe2) chore(manager): simplify backup API, dropping extension requirement ([#588](https://github.com/iotaledger/wallet.rs/pull/588)) on 2021-04-26
  - [15d9c6f4](https://github.com/iotaledger/wallet.rs/commit/15d9c6f4364de10bb91257b99916cd73f49aa6a3) Apply Version Updates From Current Changes ([#586](https://github.com/iotaledger/wallet.rs/pull/586)) on 2021-04-26
  - [08b5a41b](https://github.com/iotaledger/wallet.rs/commit/08b5a41b69c11c3eae1121cddd5f52d07a86ebf2) Fix backup restore ([#1349](https://github.com/iotaledger/wallet.rs/pull/1349)) on 2022-08-11

## \[2.0.2-alpha.13]

- Use HexEncodedAmount type in NativeTokenOptions.
  Add `localPow` and `fallbackToLocalPow` fields to ClientOptions.
  - [dc018364](https://github.com/iotaledger/wallet.rs/commit/dc018364e2e3dd9592b9d88dd4f5284e8bd256b8) Update nodejs types ([#1326](https://github.com/iotaledger/wallet.rs/pull/1326)) on 2022-08-09
- Updated to renamed query parameters. This is a breaking change.
  Renamed getLedgerStatus to getLedgerNanoStatus.
  - [d5f76ab4](https://github.com/iotaledger/wallet.rs/commit/d5f76ab4fb2f53313430b1679e70548e924d4396) Fix file name ending ([#1347](https://github.com/iotaledger/wallet.rs/pull/1347)) on 2022-08-10

## \[2.0.2-alpha.12]

- Add burning methods.
  - [cbd74771](https://github.com/iotaledger/wallet.rs/commit/cbd747715567c1c50bc265265ae14278e79800b8) Expose burning methods ([#1338](https://github.com/iotaledger/wallet.rs/pull/1338)) on 2022-08-04
- Update iota client for more logs and update @iota/types to fix types.
  - [a100c24f](https://github.com/iotaledger/wallet.rs/commit/a100c24f3448aa294a7fee640ee34df89425e200) Update iota-client and @iota/types ([#1339](https://github.com/iotaledger/wallet.rs/pull/1339)) on 2022-08-04

## \[2.0.2-alpha.11]

- Rename PoW to Pow and HRP to Hrp.
  - [9561ccd4](https://github.com/iotaledger/wallet.rs/commit/9561ccd465dc9ed593bc0dbdb8ee284d0e1e5a82) Rename PoW to Pow and HRP to Hrp ([#1329](https://github.com/iotaledger/wallet.rs/pull/1329)) on 2022-07-27

## \[2.0.2-alpha.10]

- Bumped dependencies.
  - [766ed17b](https://github.com/iotaledger/wallet.rs/commit/766ed17b50586b56af5ce4bd997a2b79112fc103) Fix workflow ([#1323](https://github.com/iotaledger/wallet.rs/pull/1323)) on 2022-07-26

## \[2.0.2-alpha.9]

- Fixed LedgerNanoSecretManager interface.
  Fixed error message when error message is not an object.
  - [ef8fea18](https://github.com/iotaledger/wallet.rs/commit/ef8fea187b714c54d66760d3717dc94eb2ba335d) Fix LedgerNanoSecretManager and handle errors that aren't an object ([#1319](https://github.com/iotaledger/wallet.rs/pull/1319)) on 2022-07-25
- Change metadata fields to hex encoded strings.
  Add incomingTransactions field to AccountMeta.
  - [a0e9cc86](https://github.com/iotaledger/wallet.rs/commit/a0e9cc86c9e8741b0d960aec634a158e464a9e6a) Change metadata fields to hex encoded strings. ([#1318](https://github.com/iotaledger/wallet.rs/pull/1318)) on 2022-07-23

## \[2.0.2-alpha.8]

- Update types with missing properties
  - [6e39cff6](https://github.com/iotaledger/wallet.rs/commit/6e39cff698a53cc43f16bc3dcfafac0dda3f8e53) feat: add missing types ([#1311](https://github.com/iotaledger/wallet.rs/pull/1311)) on 2022-07-20

## \[2.0.2-alpha.7]

- - Include @iota/types in dependencies.
  - [61b06dfe](https://github.com/iotaledger/wallet.rs/commit/61b06dfe5e4e6e15fa9fb41ad051782a2998b493) Include @iota/types in dependencies ([#1307](https://github.com/iotaledger/wallet.rs/pull/1307)) on 2022-07-18

## \[2.0.2-alpha.6]

- Added optional note to the TransactionOptions.
  - [32c01833](https://github.com/iotaledger/wallet.rs/commit/32c0183341750056b03c8346559ff11356334caa) Add `note` field ([#1295](https://github.com/iotaledger/wallet.rs/pull/1295)) on 2022-07-14
- Rename `id` property in `NativeTokenBalance` to `tokenId`
  - [6063bfaf](https://github.com/iotaledger/wallet.rs/commit/6063bfaf650318e6489923421f3c6372f8337698) chore: release latest to npm ([#1212](https://github.com/iotaledger/wallet.rs/pull/1212)) on 2022-06-28
  - [966ef5c2](https://github.com/iotaledger/wallet.rs/commit/966ef5c256d773680e35f2ce38acd035e6913517) apply version updates ([#1216](https://github.com/iotaledger/wallet.rs/pull/1216)) on 2022-06-28
  - [8442dc2c](https://github.com/iotaledger/wallet.rs/commit/8442dc2cce3d395127a279f48955f2ce0b9bc35a) Rename `id` to `tokenId` for native tokens ([#1298](https://github.com/iotaledger/wallet.rs/pull/1298)) on 2022-07-15

## \[2.0.2-alpha.5]

- Added transaction id to transaction type.
  - [c1b96c49](https://github.com/iotaledger/wallet.rs/commit/c1b96c491d1c6ac4c5d71ec8b746db64633700e5) Add missing transaction id ([#1285](https://github.com/iotaledger/wallet.rs/pull/1285)) on 2022-07-12

## \[2.0.2-alpha.4]

- Fix fields in output data types ([#1279](https://github.com/iotaledger/wallet.rs/pull/1279))
- Remove transaction option to skip sync ([#1271](https://github.com/iotaledger/wallet.rs/pull/1271))
- Add sync options for recovering accounts ([#1269](https://github.com/iotaledger/wallet.rs/pull/1269))
  - [a10ff091](https://github.com/iotaledger/wallet.rs/commit/a10ff091399200ef8c55d7f744a976206457bce5) chore: update NodeJS bindings ([#1280](https://github.com/iotaledger/wallet.rs/pull/1280)) on 2022-07-08

## \[2.0.2-alpha.3]

- Use new stronghold version
- Update balance structure
- Transaction ID under Transaction object
- Get foundry output by native token ID
- [f636777f](https://github.com/iotaledger/wallet.rs/commit/f636777f7293a3c0232877ebc9710212aeca9228) chore: update wallet.rs npm package ([#1273](https://github.com/iotaledger/wallet.rs/pull/1273)) on 2022-07-07

## \[2.0.2-alpha.2]

- Fix syncing account logic for bug around new accounts
- Add syncing option for only basic outputs with AddressUnlockCondition
- Return entire transaction object on send
- Add option to sync incoming transactions
- Add Ledger nano support for NodeJS bindings
- [461b5f75](https://github.com/iotaledger/wallet.rs/commit/461b5f757d0db64454442a8293d436ae334d14af) chore: add new changelog file ([#1262](https://github.com/iotaledger/wallet.rs/pull/1262)) on 2022-07-05

## \[2.0.2-alpha.1]

- Added typescript declaration files to the package
- Added coinType to account manager
  - [4918c237](https://github.com/iotaledger/wallet.rs/commit/4918c2377b1569d82034197f121df74fef2c583b) fix: added new file to trigger covector release ([#1240](https://github.com/iotaledger/wallet.rs/pull/1240)) on 2022-06-30
  - [617f6868](https://github.com/iotaledger/wallet.rs/commit/617f68685d759455ce25690b3ca73983b3d9a631) fix: add yarn before tsc ([#1248](https://github.com/iotaledger/wallet.rs/pull/1248)) on 2022-07-01
  - [205caf99](https://github.com/iotaledger/wallet.rs/commit/205caf99c90d0f5376fa253d9bfae5f10aab3d77) fix: trigger covector ([#1250](https://github.com/iotaledger/wallet.rs/pull/1250)) on 2022-07-01

## \[2.0.2-alpha.0]

- Pre-release of the Stardust bindings of Wallet.rs for Node.JS
  - [615f60e4](https://github.com/iotaledger/wallet.rs/commit/615f60e44f3ff4eac0270e458a42f2c28355ae04) chore: prerelease on npm ([#1228](https://github.com/iotaledger/wallet.rs/pull/1228)) on 2022-06-29

## \[1.0.4]

- Use default target features from `rustc` to determine SSE inclusion for RocksDB
  - [72327a47](https://github.com/iotaledger/wallet.rs/commit/72327a470cf0c57d930a4769f18d5e2dac857485) fix: Use default target features for RocksDB SSE inclusion ([#797](https://github.com/iotaledger/wallet.rs/pull/797)) on 2021-11-25

## \[1.0.3]

- Don't retry messages without a transaction payload.
  - [8950cc58](https://github.com/iotaledger/wallet.rs/commit/8950cc5844279d5656cc014ce31b9e6eb3be7068) fix: don't retry message without a transaction payload ([#795](https://github.com/iotaledger/wallet.rs/pull/795)) on 2021-11-19

## \[1.0.2]

- Fix build scripts and workflows
  - [f0a39030](https://github.com/iotaledger/wallet.rs/commit/f0a39030974ecda65f1f6f9622e1e4991cba1c60) ci: Fix Node.js bindings scripts and workflows ([#752](https://github.com/iotaledger/wallet.rs/pull/752)) on 2021-09-01

## \[1.0.1]

- Fix workflow to prebuild binaries
  - [46442719](https://github.com/iotaledger/wallet.rs/commit/46442719bd9aed7e7d70133fb37fc9fe7fd855e4) Nodejs binding release workflow ([#749](https://github.com/iotaledger/wallet.rs/pull/749)) on 2021-08-31

## \[1.0.0]

- Update to newer neon version with napi-6 to allow concurrent function calls
  Move event listener functions to the AccountManager
  - [b41a2afc](https://github.com/iotaledger/wallet.rs/commit/b41a2afcc8a55440601811f518e98d58f1c51aad) New nodejs binding using the wallet message system and latest neon version. ([#674](https://github.com/iotaledger/wallet.rs/pull/674)) on 2021-08-31
  - [3b2c8431](https://github.com/iotaledger/wallet.rs/commit/3b2c843197556e6725442b74cbd44ffce88357bd) apply version updates ([#746](https://github.com/iotaledger/wallet.rs/pull/746)) on 2021-08-31
  - [c77bf7c5](https://github.com/iotaledger/wallet.rs/commit/c77bf7c57d1833d86edf34e48a158d6b638bac88) Fix workflow path ([#747](https://github.com/iotaledger/wallet.rs/pull/747)) on 2021-08-31

## \[0.6.0]

- Fixes edge case in account discovery.
  - [2320748d](https://github.com/iotaledger/wallet.rs/commit/2320748d968ca634e2e321ff6bcfe10500887a67) fix account discovery when there is only an internal address with balance ([#672](https://github.com/iotaledger/wallet.rs/pull/672)) on 2021-06-28
- Added GetBalance to apiTimeouts.
  - [3db454e2](https://github.com/iotaledger/wallet.rs/commit/3db454e26131d5f8706e0b4ee0f4390a77416229) add GetBalance to API timeouts ([#667](https://github.com/iotaledger/wallet.rs/pull/667)) on 2021-07-19
- Added `mqttDisabled` option to disable mqtt.
  - [349f8307](https://github.com/iotaledger/wallet.rs/commit/349f83074a378ca228dd86c3c975411de9b184fe) Add option to disable mqtt ([#665](https://github.com/iotaledger/wallet.rs/pull/665)) on 2021-06-22
- Add optional OutputKind for transfers to enable the creation of dust allowance outputs.
  consolidateOutputs() has also an optional boolean to define if dust outputs should also get consolidated.
  - [6eea2a71](https://github.com/iotaledger/wallet.rs/commit/6eea2a71da14fb2e0ad2e0991d6bf07c07ce37e0) Add dust allowance outputs support ([#678](https://github.com/iotaledger/wallet.rs/pull/678)) on 2021-07-19

## \[0.5.0]

- Added generateAddresses function.
  - [ee3c0fa0](https://github.com/iotaledger/wallet.rs/commit/ee3c0fa0ae12cf80161d351a9f0af83c7c49f4a6) Add generateAddresses change file ([#660](https://github.com/iotaledger/wallet.rs/pull/660)) on 2021-06-11
- Added primaryNode and primaryPoWNode to the ClientOptions.
  - [3d66485c](https://github.com/iotaledger/wallet.rs/commit/3d66485ca11d21fbd64fafec9e68b377235c8c9b) Bindings/primary node ([#629](https://github.com/iotaledger/wallet.rs/pull/629)) on 2021-06-10
- Added startBackgroundSync.
  - [bd44d4b0](https://github.com/iotaledger/wallet.rs/commit/bd44d4b04c46f6560404761615f78ba36774d726) Expose start_background_sync ([#640](https://github.com/iotaledger/wallet.rs/pull/640)) on 2021-06-07
- Improve syncing speed.
  - [72e6d649](https://github.com/iotaledger/wallet.rs/commit/72e6d6493ae497172190300b8da8cdecd5d47d52) improve syncing speed ([#648](https://github.com/iotaledger/wallet.rs/pull/648)) on 2021-06-10
- Build bindings on Ubuntu 18.04 to support older versions of glibc
  - [359eed9c](https://github.com/iotaledger/wallet.rs/commit/359eed9c42e5e8f92f215b9d3a724b85e1837a87) fix(ci): Build Node.js bindings on Ubuntu 18.04 ([#636](https://github.com/iotaledger/wallet.rs/pull/636)) on 2021-06-10

## \[0.4.2]

- Improve syncing speed.
  - [3b51dd95](https://github.com/iotaledger/wallet.rs/commit/3b51dd95998968867655fe2c4ec44d41aa252178) Improve syncing ([#633](https://github.com/iotaledger/wallet.rs/pull/633)) on 2021-05-27
  - [c39362f3](https://github.com/iotaledger/wallet.rs/commit/c39362f3f2ea2975754a623884d8d7a7ae09ce6c) apply version updates ([#628](https://github.com/iotaledger/wallet.rs/pull/628)) on 2021-05-27
  - [e09167ae](https://github.com/iotaledger/wallet.rs/commit/e09167ae97493961980d5ab9f8d448ae46c53799) improve syncing speed and add logger example ([#638](https://github.com/iotaledger/wallet.rs/pull/638)) on 2021-06-02

## \[0.4.1]

- Added optional remainder property in BalanceChangeEvent.
  - [a8bb9306](https://github.com/iotaledger/wallet.rs/commit/a8bb9306861bb7e965354ce3c94e6de2df5e28fd) add remainder property in BalanceChangeEvent ([#627](https://github.com/iotaledger/wallet.rs/pull/627)) on 2021-05-21
- Change input selection to not always use all outputs from an address, but only the required ones.
  - [bc977be6](https://github.com/iotaledger/wallet.rs/commit/bc977be636261bbd1dc0da0d42ce7048343960aa) Change input selection ([#424](https://github.com/iotaledger/wallet.rs/pull/424)) on 2021-05-21
- Added skipPolling and pollingInterval options to the ManagerOptions.
  - [58dda772](https://github.com/iotaledger/wallet.rs/commit/58dda7726e2c728a81faff4316c9dd14357c4d44) add skipPolling and pollingInterval options ([#630](https://github.com/iotaledger/wallet.rs/pull/630)) on 2021-05-27
- Send sync requests in chunks to prevent timeouts, make background sync not blocking the whole time.
  Changed polling interval to wait after each sync operations, so it doesn't start immediately if the syncing takes longer than the polling interval.
  - [3b51dd95](https://github.com/iotaledger/wallet.rs/commit/3b51dd95998968867655fe2c4ec44d41aa252178) Improve syncing ([#633](https://github.com/iotaledger/wallet.rs/pull/633)) on 2021-05-27

## \[0.4.0]

- Websocket is used as default now and new fields are added to the BrokerOptions.
  - [b7c74521](https://github.com/iotaledger/wallet.rs/commit/b7c74521cb6cb6126d3c8338c74132ad40d6ff23) add changes files ([#626](https://github.com/iotaledger/wallet.rs/pull/626)) on 2021-05-20
- Added auth options to getNodeInfo.
  - [b7c74521](https://github.com/iotaledger/wallet.rs/commit/b7c74521cb6cb6126d3c8338c74132ad40d6ff23) add changes files ([#626](https://github.com/iotaledger/wallet.rs/pull/626)) on 2021-05-20
- Accept client options only with url instead of node object also for the manager.
  - [ba4e3b66](https://github.com/iotaledger/wallet.rs/commit/ba4e3b669599510faceed6bcc9465124e0a77f2b) manager accept same client options as the account ([#622](https://github.com/iotaledger/wallet.rs/pull/622)) on 2021-05-18

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
  - [991c2e6](https://github.com/iotaledger/wallet.rs/commit/991c2e68c1f88f0c327d1cd37a1275089aaf0ed3) fix(stronghold): mark client as loaded if the snapshot decrypt succeeded ([#357](https://github.com/iotaledger/wallet.rs/pull/357)) on 2021-03-01
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
