# Interface: AccountSyncOptions

Sync options for an account

## Table of contents

### Properties

- [addresses](AccountSyncOptions.md#addresses)
- [addressStartIndex](AccountSyncOptions.md#addressstartindex)
- [addressStartIndexInternal](AccountSyncOptions.md#addressstartindexinternal)
- [forceSyncing](AccountSyncOptions.md#forcesyncing)
- [syncPendingTransactions](AccountSyncOptions.md#syncpendingtransactions)
- [syncAliasesAndNfts](AccountSyncOptions.md#syncaliasesandnfts)
- [syncOnlyMostBasicOutputs](AccountSyncOptions.md#synconlymostbasicoutputs)
- [syncNativeTokenFoundries](AccountSyncOptions.md#syncnativetokenfoundries)

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

### addressStartIndexInternal

• `Optional` **addressStartIndexInternal**: `number`

Address index from which to start syncing internal addresses. 0 by default, using a higher index will be faster
because addresses with a lower index will be skipped, but could result in a wrong balance for that reason

___

### forceSyncing

• `Optional` **forceSyncing**: `boolean`

Usually syncing is skipped if it's called in between 200ms, because there can only be new changes every
milestone and calling it twice "at the same time" will not return new data
When this to true, we will sync anyways, even if it's called 0ms after the las sync finished. Default: false.

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

___

### syncNativeTokenFoundries

• `Optional` **syncNativeTokenFoundries**: `boolean`

Sync native token foundries, so their metadata can be returned in the balance. Default: false.
