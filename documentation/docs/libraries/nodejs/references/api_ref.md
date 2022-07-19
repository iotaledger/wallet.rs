# @iota/wallet

## Table of contents

### Classes

- [Account](classes/Account.md)
- [AccountManager](classes/AccountManager.md)

### Functions

- [initLogger](api_ref.md#initlogger)

### Type Aliases

- [AccountId](api_ref.md#accountid)

### Interfaces

- [LoggerConfig](interfaces/LoggerConfig.md)
- [NodeInfoWrapper](interfaces/NodeInfoWrapper.md)
- [OutputData](interfaces/OutputData.md)
- [PreparedTransactionData](interfaces/PreparedTransactionData.md)
- [InputSigningData](interfaces/InputSigningData.md)
- [RemainderData](interfaces/RemainderData.md)

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
