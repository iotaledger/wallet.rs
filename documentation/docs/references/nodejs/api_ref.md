# @iota/wallet

## Table of contents

### Classes

- [Account](classes/Account.md)
- [AccountManager](classes/AccountManager.md)

### Functions

- [initLogger](api_ref.md#initlogger)

### Type Aliases

- [AccountId](api_ref.md#accountid)
- [EventType](api_ref.md#eventtype)
- [Auth](api_ref.md#auth)
- [Node](api_ref.md#node)
- [SecretManager](api_ref.md#secretmanager)
- [RemainderValueStrategy](api_ref.md#remaindervaluestrategy)
- [ChangeAddress](api_ref.md#changeaddress)
- [ReuseAddress](api_ref.md#reuseaddress)
- [CustomAddress](api_ref.md#customaddress)

### Interfaces

- [AccountBalance](interfaces/AccountBalance.md)
- [AccountSyncOptions](interfaces/AccountSyncOptions.md)
- [AccountMeta](interfaces/AccountMeta.md)
- [BaseCoinBalance](interfaces/BaseCoinBalance.md)
- [NativeTokenBalance](interfaces/NativeTokenBalance.md)
- [CreateAccountPayload](interfaces/CreateAccountPayload.md)
- FilterOptions
- [AccountManagerOptions](interfaces/AccountManagerOptions.md)
- [Address](interfaces/Address.md)
- [AddressWithAmount](interfaces/AddressWithAmount.md)
- [AddressWithUnspentOutputs](interfaces/AddressWithUnspentOutputs.md)
- [AddressWithMicroAmount](interfaces/AddressWithMicroAmount.md)
- [AddressNativeTokens](interfaces/AddressNativeTokens.md)
- [AddressNftId](interfaces/AddressNftId.md)
- [AddressGenerationOptions](interfaces/AddressGenerationOptions.md)
- [GenerateAddressMetadata](interfaces/GenerateAddressMetadata.md)
- [BuildAliasOutputData](interfaces/BuildAliasOutputData.md)
- [BuildBasicOutputData](interfaces/BuildBasicOutputData.md)
- [BuildFoundryOutputData](interfaces/BuildFoundryOutputData.md)
- [BuildNftOutputData](interfaces/BuildNftOutputData.md)
- [LoggerConfig](interfaces/LoggerConfig.md)
- [NetworkInfo](interfaces/NetworkInfo.md)
- [ClientOptions](interfaces/ClientOptions.md)
- [NodeInfoWrapper](interfaces/NodeInfoWrapper.md)
- [OutputData](interfaces/OutputData.md)
- [Segment](interfaces/Segment.md)
- [OutputOptions](interfaces/OutputOptions.md)
- [Assets](interfaces/Assets.md)
- [Features](interfaces/Features.md)
- [Unlocks](interfaces/Unlocks.md)
- [StorageDeposit](interfaces/StorageDeposit.md)
- [PreparedTransactionData](interfaces/PreparedTransactionData.md)
- [InputSigningData](interfaces/InputSigningData.md)
- [RemainderData](interfaces/RemainderData.md)
- [LedgerNanoSecretManager](interfaces/LedgerNanoSecretManager.md)
- [MnemonicSecretManager](interfaces/MnemonicSecretManager.md)
- [SeedSecretManager](interfaces/SeedSecretManager.md)
- [StrongholdSecretManager](interfaces/StrongholdSecretManager.md)
- [LedgerNanoStatus](interfaces/LedgerNanoStatus.md)
- [LedgerApp](interfaces/LedgerApp.md)
- [SignedTransactionEssence](interfaces/SignedTransactionEssence.md)
- [Transaction](interfaces/Transaction.md)
- [MintTokenTransaction](interfaces/MintTokenTransaction.md)
- [TransactionOptions](interfaces/TransactionOptions.md)
- [IncreaseNativeTokenSupplyOptions](interfaces/IncreaseNativeTokenSupplyOptions.md)
- [NativeTokenOptions](interfaces/NativeTokenOptions.md)
- [NftOptions](interfaces/NftOptions.md)

### Enumerations

- [CoinType](enums/CoinType.md)
- [AddressType](enums/AddressType.md)
- [WalletEvent](enums/WalletEvent.md)
- [Network](enums/Network.md)
- [OutputsToClaim](enums/OutputsToClaim.md)
- [ReturnStrategy](enums/ReturnStrategy.md)
- [LedgerDeviceType](enums/LedgerDeviceType.md)
- [InclusionState](enums/InclusionState.md)

## Functions

### initLogger

▸ **initLogger**(`config`): `any`

Function to create wallet logs

#### Parameters

| Name | Type |
| :------ | :------ |
| `config` | [`LoggerConfig`](interfaces/LoggerConfig.md) |

#### Returns

`any`

## Type Aliases

### AccountId

Ƭ **AccountId**: `number` \| `string`

Account identifier
Could be the account index (number) or account alias (string)

___

### EventType

Ƭ **EventType**: ``"*"`` \| ``"ErrorThrown"`` \| ``"ConsolidationRequired"`` \| ``"LedgerAddressGeneration"`` \| ``"NewOutput"`` \| ``"SpentOutput"`` \| ``"TransactionInclusion"`` \| ``"TransactionProgress"``

Wallet event types

___

### Auth

Ƭ **Auth**: `Object`

Basic Auth or JWT

#### Type declaration

| Name | Type |
| :------ | :------ |
| `jwt?` | `string` |
| `username?` | `string` |
| `password?` | `string` |

___

### Node

Ƭ **Node**: `Object`

A node object for the client

#### Type declaration

| Name | Type |
| :------ | :------ |
| `url` | `string` |
| `auth?` | [`Auth`](api_ref.md#auth) |
| `disabled?` | `boolean` |

___

### SecretManager

Ƭ **SecretManager**: [`LedgerNanoSecretManager`](interfaces/LedgerNanoSecretManager.md) \| [`MnemonicSecretManager`](interfaces/MnemonicSecretManager.md) \| [`StrongholdSecretManager`](interfaces/StrongholdSecretManager.md)

Supported secret managers

___

### RemainderValueStrategy

Ƭ **RemainderValueStrategy**: [`ChangeAddress`](api_ref.md#changeaddress) \| [`ReuseAddress`](api_ref.md#reuseaddress) \| [`CustomAddress`](api_ref.md#customaddress)

The RemainderValueStrategy

___

### ChangeAddress

Ƭ **ChangeAddress**: `Object`

ChangeAddress variant of RemainderValueStrategy

#### Type declaration

| Name | Type |
| :------ | :------ |
| `strategy` | ``"ChangeAddress"`` |
| `value` | ``null`` |

___

### ReuseAddress

Ƭ **ReuseAddress**: `Object`

ReuseAddress variant of RemainderValueStrategy

#### Type declaration

| Name | Type |
| :------ | :------ |
| `strategy` | ``"ReuseAddress"`` |
| `value` | ``null`` |

___

### CustomAddress

Ƭ **CustomAddress**: `Object`

CustomAddress variant of RemainderValueStrategy

#### Type declaration

| Name | Type |
| :------ | :------ |
| `strategy` | ``"CustomAddress"`` |
| `value` | `string` |
