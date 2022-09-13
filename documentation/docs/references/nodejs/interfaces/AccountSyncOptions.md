# Interface: AccountSyncOptions

Sync options for an account

## Table of contents

### Properties

- [addresses](AccountSyncOptions.md#addresses)
- [addressStartIndex](AccountSyncOptions.md#addressstartindex)
- [forceSyncing](AccountSyncOptions.md#forcesyncing)
- [syncPendingTransactions](AccountSyncOptions.md#syncpendingtransactions)
- [syncAliasesAndNfts](AccountSyncOptions.md#syncaliasesandnfts)
- [syncOnlyMostBasicOutputs](AccountSyncOptions.md#synconlymostbasicoutputs)

## Properties

### addresses

• `Optional` **addresses**: `string`[]

Specific Bech32 encoded addresses of the account to sync, if addresses are provided,
then `address_start_index` will be ignored

___

### addressStartIndex

• `Optional` **addressStartIndex**: `number`

Address index from which to start syncing addresses. 0 by default, using a higher index will be faster because
addresses with a lower index will be skipped, but could result in a wrong balance for that reason

___

### forceSyncing

• `Optional` **forceSyncing**: `boolean`

Usually we skip syncing if it's called within a few seconds, because there can only be new changes every 5
seconds. But if we change the client options, we need to resync, because the new node could be from a nother
network and then we need to check all addresses. This will also ignore `address_start_index` and sync all
addresses. Default: false.

___

### syncPendingTransactions

• `Optional` **syncPendingTransactions**: `boolean`

Checks pending transactions and promotes/reattaches them if necessary.  Default: true.

___

### syncAliasesAndNfts

• `Optional` **syncAliasesAndNfts**: `boolean`

Specifies if only basic outputs should be synced or also alias and nft outputs. Default: true.

___

### syncOnlyMostBasicOutputs

• `Optional` **syncOnlyMostBasicOutputs**: `boolean`

Specifies if only basic outputs with an AddressUnlockCondition alone should be synced, will overwrite
`syncAliasesAndNfts`. Default: false.
