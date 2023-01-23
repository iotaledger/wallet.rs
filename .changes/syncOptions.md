---
"nodejs-binding": patch
---

Renamed AccountSyncOptions to SyncOptions;
Added AccountSyncOptions, AliasSyncOptions and NftSyncOptions;
Replaced SyncOptions::syncAliasesAndNfts with SyncOptions::{account, alias, nft};
