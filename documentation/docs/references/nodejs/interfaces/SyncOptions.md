# Interface: SyncOptions

Sync options for an account

## Table of contents

### Properties

- [addresses](SyncOptions.md#addresses)
- [addressStartIndex](SyncOptions.md#addressstartindex)
- [addressStartIndexInternal](SyncOptions.md#addressstartindexinternal)
- [forceSyncing](SyncOptions.md#forcesyncing)
- [syncPendingTransactions](SyncOptions.md#syncpendingtransactions)
- [account](SyncOptions.md#account)
- [alias](SyncOptions.md#alias)
- [nft](SyncOptions.md#nft)
- [syncOnlyMostBasicOutputs](SyncOptions.md#synconlymostbasicoutputs)
- [syncNativeTokenFoundries](SyncOptions.md#syncnativetokenfoundries)

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

### account

• `Optional` **account**: [`AccountSyncOptions`](AccountSyncOptions.md)

Specifies what outputs should be synced for the ed25519 addresses from the account.

___

### alias

• `Optional` **alias**: [`AliasSyncOptions`](AliasSyncOptions.md)

Specifies what outputs should be synced for the address of an alias output.

___

### nft

• `Optional` **nft**: [`NftSyncOptions`](NftSyncOptions.md)

Specifies what outputs should be synced for the address of an nft output.

___

### syncOnlyMostBasicOutputs

• `Optional` **syncOnlyMostBasicOutputs**: `boolean`

Specifies if only basic outputs with an AddressUnlockCondition alone should be synced, will overwrite `account`, `alias` and `nft` options.  Default: false.

___

### syncNativeTokenFoundries

• `Optional` **syncNativeTokenFoundries**: `boolean`

Sync native token foundries, so their metadata can be returned in the balance. Default: false.
