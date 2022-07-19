# Class: AccountManager

The AccountManager class.

## Table of contents

### Methods

- [backup](AccountManager.md#backup)
- [bech32ToHex](AccountManager.md#bech32tohex)
- [changeStrongholdPassword](AccountManager.md#changestrongholdpassword)
- [clearStrongholdPassword](AccountManager.md#clearstrongholdpassword)
- [createAccount](AccountManager.md#createaccount)
- [deleteAccountsAndDatabase](AccountManager.md#deleteaccountsanddatabase)
- [destroy](AccountManager.md#destroy)
- [emitTestEvent](AccountManager.md#emittestevent)
- [generateMnemonic](AccountManager.md#generatemnemonic)
- [getAccount](AccountManager.md#getaccount)
- [getAccounts](AccountManager.md#getaccounts)
- [getNodeInfo](AccountManager.md#getnodeinfo)
- [getLedgerStatus](AccountManager.md#getledgerstatus)
- [hexToBech32](AccountManager.md#hextobech32)
- [isStrongholdPasswordAvailable](AccountManager.md#isstrongholdpasswordavailable)
- [listen](AccountManager.md#listen)
- [clearListeners](AccountManager.md#clearlisteners)
- [recoverAccounts](AccountManager.md#recoveraccounts)
- [removeLatestAccount](AccountManager.md#removelatestaccount)
- [restoreBackup](AccountManager.md#restorebackup)
- [setClientOptions](AccountManager.md#setclientoptions)
- [setStrongholdPassword](AccountManager.md#setstrongholdpassword)
- [setStrongholdPasswordClearInterval](AccountManager.md#setstrongholdpasswordclearinterval)
- [startBackgroundSync](AccountManager.md#startbackgroundsync)
- [stopBackgroundSync](AccountManager.md#stopbackgroundsync)
- [storeMnemonic](AccountManager.md#storemnemonic)
- [verifyMnemonic](AccountManager.md#verifymnemonic)

## Methods

### backup

▸ **backup**(`destination`, `password`): `Promise`<`void`\>

Backup the data to a Stronghold snapshot.

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |
| `password` | `string` |

#### Returns

`Promise`<`void`\>

___

### bech32ToHex

▸ **bech32ToHex**(`bech32Address`): `Promise`<`string`\>

Transform a bech32 encoded address to a hex encoded address

#### Parameters

| Name | Type |
| :------ | :------ |
| `bech32Address` | `string` |

#### Returns

`Promise`<`string`\>

___

### changeStrongholdPassword

▸ **changeStrongholdPassword**(`currentPassword`, `newPassword`): `Promise`<`void`\>

Change the Stronghold password.

#### Parameters

| Name | Type |
| :------ | :------ |
| `currentPassword` | `string` |
| `newPassword` | `string` |

#### Returns

`Promise`<`void`\>

___

### clearStrongholdPassword

▸ **clearStrongholdPassword**(): `Promise`<`void`\>

Clear the Stronghold password from memory.

#### Returns

`Promise`<`void`\>

___

### createAccount

▸ **createAccount**(`payload`): `Promise`<[`Account`](Account.md)\>

Create a new account.

#### Parameters

| Name | Type |
| :------ | :------ |
| `payload` | `CreateAccountPayload` |

#### Returns

`Promise`<[`Account`](Account.md)\>

___

### deleteAccountsAndDatabase

▸ **deleteAccountsAndDatabase**(): `Promise`<`void`\>

Delete all accounts and the database folder.

#### Returns

`Promise`<`void`\>

___

### destroy

▸ **destroy**(): `void`

Destroy the AccountManager and drop its database connection.

#### Returns

`void`

___

### emitTestEvent

▸ **emitTestEvent**(`event`): `Promise`<`void`\>

Emit a provided event for testing of the event system.

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `WalletEvent` |

#### Returns

`Promise`<`void`\>

___

### generateMnemonic

▸ **generateMnemonic**(): `Promise`<`string`\>

Generate a random BIP39 mnemonic.

#### Returns

`Promise`<`string`\>

___

### getAccount

▸ **getAccount**(`accountId`): `Promise`<[`Account`](Account.md)\>

Get an account by its alias or index.

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountId` | [`AccountId`](../api_ref.md#accountid) |

#### Returns

`Promise`<[`Account`](Account.md)\>

___

### getAccounts

▸ **getAccounts**(): `Promise`<[`Account`](Account.md)[]\>

Get all accounts.

#### Returns

`Promise`<[`Account`](Account.md)[]\>

___

### getNodeInfo

▸ **getNodeInfo**(`url?`, `auth?`): `Promise`<[`NodeInfoWrapper`](../interfaces/NodeInfoWrapper.md)\>

Get the node info.

#### Parameters

| Name | Type |
| :------ | :------ |
| `url?` | `string` |
| `auth?` | `Auth` |

#### Returns

`Promise`<[`NodeInfoWrapper`](../interfaces/NodeInfoWrapper.md)\>

___

### getLedgerStatus

▸ **getLedgerStatus**(): `Promise`<`LedgerStatus`\>

Get the status for a Ledger Nano.

#### Returns

`Promise`<`LedgerStatus`\>

___

### hexToBech32

▸ **hexToBech32**(`hex`, `bech32Hrp?`): `Promise`<`string`\>

Transform hex encoded address to bech32 encoded address. If no bech32Hrp
is provided, the AccountManager will attempt to retrieve it from the
NodeInfo. If this does not succeed, it will default to the Shimmer testnet bech32Hrp.

#### Parameters

| Name | Type |
| :------ | :------ |
| `hex` | `string` |
| `bech32Hrp?` | `string` |

#### Returns

`Promise`<`string`\>

___

### isStrongholdPasswordAvailable

▸ **isStrongholdPasswordAvailable**(): `Promise`<`boolean`\>

Check if the Stronghold password has been set.

#### Returns

`Promise`<`boolean`\>

___

### listen

▸ **listen**(`eventTypes`, `callback`): `void`

Listen to wallet events with a callback. An empty array will listen to all possible events.

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventTypes` | `EventType`[] |
| `callback` | (`error`: `Error`, `result`: `string`) => `void` |

#### Returns

`void`

___

### clearListeners

▸ **clearListeners**(`eventTypes`): `void`

Clear the callbacks for provided events. An empty array will clear all listeners.

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventTypes` | `EventType`[] |

#### Returns

`void`

___

### recoverAccounts

▸ **recoverAccounts**(`accountGapLimit`, `addressGapLimit`, `syncOptions`): `Promise`<[`Account`](Account.md)[]\>

Find accounts with unspent outputs.

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountGapLimit` | `number` |
| `addressGapLimit` | `number` |
| `syncOptions` | `AccountSyncOptions` |

#### Returns

`Promise`<[`Account`](Account.md)[]\>

___

### removeLatestAccount

▸ **removeLatestAccount**(): `Promise`<`void`\>

Delete the latest account.

#### Returns

`Promise`<`void`\>

___

### restoreBackup

▸ **restoreBackup**(`source`, `password`): `Promise`<`void`\>

Restore a backup from a Stronghold snapshot.

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `string` |
| `password` | `string` |

#### Returns

`Promise`<`void`\>

___

### setClientOptions

▸ **setClientOptions**(`options`): `Promise`<`void`\>

Set ClientOptions.

#### Parameters

| Name | Type |
| :------ | :------ |
| `options` | `ClientOptions` |

#### Returns

`Promise`<`void`\>

___

### setStrongholdPassword

▸ **setStrongholdPassword**(`password`): `Promise`<`void`\>

Set the Stronghold password.

#### Parameters

| Name | Type |
| :------ | :------ |
| `password` | `string` |

#### Returns

`Promise`<`void`\>

___

### setStrongholdPasswordClearInterval

▸ **setStrongholdPasswordClearInterval**(`intervalInMilliseconds?`): `Promise`<`void`\>

Set the interval after which the Stronghold password gets cleared from memory.

#### Parameters

| Name | Type |
| :------ | :------ |
| `intervalInMilliseconds?` | `number` |

#### Returns

`Promise`<`void`\>

___

### startBackgroundSync

▸ **startBackgroundSync**(`options?`, `intervalInMilliseconds?`): `Promise`<`void`\>

Start the background syncing process for all accounts.

#### Parameters

| Name | Type |
| :------ | :------ |
| `options?` | `AccountSyncOptions` |
| `intervalInMilliseconds?` | `number` |

#### Returns

`Promise`<`void`\>

___

### stopBackgroundSync

▸ **stopBackgroundSync**(): `Promise`<`void`\>

Stop the background syncing process for all accounts.

#### Returns

`Promise`<`void`\>

___

### storeMnemonic

▸ **storeMnemonic**(`mnemonic`): `Promise`<`void`\>

Store a mnemonic in the Stronghold snapshot.

#### Parameters

| Name | Type |
| :------ | :------ |
| `mnemonic` | `string` |

#### Returns

`Promise`<`void`\>

___

### verifyMnemonic

▸ **verifyMnemonic**(`mnemonic`): `Promise`<`void`\>

Verify if a mnemonic is a valid BIP39 mnemonic.

#### Parameters

| Name | Type |
| :------ | :------ |
| `mnemonic` | `string` |

#### Returns

`Promise`<`void`\>
