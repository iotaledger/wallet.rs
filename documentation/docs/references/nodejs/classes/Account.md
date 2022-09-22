# Class: Account

The Account class.

## Table of contents

### Methods

- [buildAliasOutput](Account.md#buildaliasoutput)
- [buildBasicOutput](Account.md#buildbasicoutput)
- [buildFoundryOutput](Account.md#buildfoundryoutput)
- [buildNftOutput](Account.md#buildnftoutput)
- [burnNativeToken](Account.md#burnnativetoken)
- [burnNft](Account.md#burnnft)
- [claimOutputs](Account.md#claimoutputs)
- [consolidateOutputs](Account.md#consolidateoutputs)
- [createAliasOutput](Account.md#createaliasoutput)
- [decreaseNativeTokenSupply](Account.md#decreasenativetokensupply)
- [destroyAlias](Account.md#destroyalias)
- [destroyFoundry](Account.md#destroyfoundry)
- [generateAddress](Account.md#generateaddress)
- [generateAddresses](Account.md#generateaddresses)
- [getBalance](Account.md#getbalance)
- [getOutput](Account.md#getoutput)
- [getFoundryOutput](Account.md#getfoundryoutput)
- [getOutputsWithAdditionalUnlockConditions](Account.md#getoutputswithadditionalunlockconditions)
- [getTransaction](Account.md#gettransaction)
- [listAddresses](Account.md#listaddresses)
- [listAddressesWithUnspentOutputs](Account.md#listaddresseswithunspentoutputs)
- [listOutputs](Account.md#listoutputs)
- [listPendingTransactions](Account.md#listpendingtransactions)
- [listIncomingTransactions](Account.md#listincomingtransactions)
- [listTransactions](Account.md#listtransactions)
- [listUnspentOutputs](Account.md#listunspentoutputs)
- [getMetadata](Account.md#getmetadata)
- [minimumRequiredStorageDeposit](Account.md#minimumrequiredstoragedeposit)
- [increaseNativeTokenSupply](Account.md#increasenativetokensupply)
- [mintNativeToken](Account.md#mintnativetoken)
- [mintNfts](Account.md#mintnfts)
- [prepareOutput](Account.md#prepareoutput)
- [prepareSendAmount](Account.md#preparesendamount)
- [prepareTransaction](Account.md#preparetransaction)
- [sendAmount](Account.md#sendamount)
- [sendMicroTransaction](Account.md#sendmicrotransaction)
- [sendNativeTokens](Account.md#sendnativetokens)
- [sendNft](Account.md#sendnft)
- [sendOutputs](Account.md#sendoutputs)
- [signTransactionEssence](Account.md#signtransactionessence)
- [submitAndStoreTransaction](Account.md#submitandstoretransaction)
- [sync](Account.md#sync)

## Methods

### buildAliasOutput

▸ **buildAliasOutput**(`data`): `Promise`<`IAliasOutput`\>

Build an `AliasOutput`.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `data` | [`BuildAliasOutputData`](../interfaces/BuildAliasOutputData.md) | Options for building an `AliasOutput`. |

#### Returns

`Promise`<`IAliasOutput`\>

The built `AliasOutput`.

___

### buildBasicOutput

▸ **buildBasicOutput**(`data`): `Promise`<`IBasicOutput`\>

Build a `BasicOutput`.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `data` | [`BuildBasicOutputData`](../interfaces/BuildBasicOutputData.md) | Options for building a `BasicOutput`. |

#### Returns

`Promise`<`IBasicOutput`\>

The built `BasicOutput`.

___

### buildFoundryOutput

▸ **buildFoundryOutput**(`data`): `Promise`<`IFoundryOutput`\>

Build a `FoundryOutput`.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `data` | [`BuildFoundryOutputData`](../interfaces/BuildFoundryOutputData.md) | Options for building a `FoundryOutput`. |

#### Returns

`Promise`<`IFoundryOutput`\>

The built `FoundryOutput`.

___

### buildNftOutput

▸ **buildNftOutput**(`data`): `Promise`<`INftOutput`\>

Build an `NftOutput`.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `data` | [`BuildNftOutputData`](../interfaces/BuildNftOutputData.md) | Options for building an `NftOutput`. |

#### Returns

`Promise`<`INftOutput`\>

The built `NftOutput`.

___

### burnNativeToken

▸ **burnNativeToken**(`tokenId`, `burnAmount`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
recommended to use melting, if the foundry output is available.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `tokenId` | `string` | The native token id. |
| `burnAmount` | `string` | The to be burned amount. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### burnNft

▸ **burnNft**(`nftId`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Burn an nft output. Outputs controlled by it will be sweeped before if they don't have a storage
deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
burning, the foundry can never be destroyed anymore.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nftId` | `string` | The NftId. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### claimOutputs

▸ **claimOutputs**(`outputIds`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Claim basic or nft outputs that have additional unlock conditions
to their `AddressUnlockCondition` from the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputIds` | `string`[] | The outputs to claim. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The resulting transaction.

___

### consolidateOutputs

▸ **consolidateOutputs**(`force`, `outputConsolidationThreshold?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Consolidate basic outputs with only an `AddressUnlockCondition` from an account
by sending them to an own address again if the output amount is greater or
equal to the output consolidation threshold.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `force` | `boolean` | Force consolidation on addresses where the threshold isn't met. |
| `outputConsolidationThreshold?` | `number` | A default threshold is used if this is omitted. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The consolidation transaction.

___

### createAliasOutput

▸ **createAliasOutput**(`aliasOutputOptions?`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

`createAliasOutput` creates an alias output

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `aliasOutputOptions?` | [`AliasOutputOptions`](../interfaces/AliasOutputOptions.md) | The alias output options. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

A transaction object.

___

### decreaseNativeTokenSupply

▸ **decreaseNativeTokenSupply**(`tokenId`, `meltAmount`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Melt native tokens. This happens with the foundry output which minted them, by increasing its
`melted_tokens` field.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `tokenId` | `string` | The native token id. |
| `meltAmount` | `string` | To be melted amount. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### destroyAlias

▸ **destroyAlias**(`aliasId`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Destroy an alias output. Outputs controlled by it will be sweeped before if they don't have a
storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
sent to the governor address.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `aliasId` | `string` | The AliasId. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### destroyFoundry

▸ **destroyFoundry**(`foundryId`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Function to destroy a foundry output with a circulating supply of 0.
Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `foundryId` | `string` | The FoundryId. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### generateAddress

▸ **generateAddress**(`options?`): `Promise`<[`Address`](../interfaces/Address.md)\>

Generate a new unused address.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options?` | [`AddressGenerationOptions`](../interfaces/AddressGenerationOptions.md) | Options for address generation. |

#### Returns

`Promise`<[`Address`](../interfaces/Address.md)\>

The address.

___

### generateAddresses

▸ **generateAddresses**(`amount`, `options?`): `Promise`<[`Address`](../interfaces/Address.md)[]\>

Generate new unused addresses.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `amount` | `number` | The amount of addresses to generate. |
| `options?` | [`AddressGenerationOptions`](../interfaces/AddressGenerationOptions.md) | Options for address generation. |

#### Returns

`Promise`<[`Address`](../interfaces/Address.md)[]\>

The addresses.

___

### getBalance

▸ **getBalance**(): `Promise`<[`AccountBalance`](../interfaces/AccountBalance.md)\>

Get the account balance.

#### Returns

`Promise`<[`AccountBalance`](../interfaces/AccountBalance.md)\>

The account balance.

___

### getOutput

▸ **getOutput**(`outputId`): `Promise`<[`OutputData`](../interfaces/OutputData.md)\>

Get the data for an output.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputId` | `string` | The output to get. |

#### Returns

`Promise`<[`OutputData`](../interfaces/OutputData.md)\>

The `OutputData`.

___

### getFoundryOutput

▸ **getFoundryOutput**(`tokenId`): `Promise`<`IFoundryOutput`\>

Get a `FoundryOutput` by native token ID. It will try to get the foundry from
the account, if it isn't in the account it will try to get it from the node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `tokenId` | `string` | The native token ID to get the foundry for. |

#### Returns

`Promise`<`IFoundryOutput`\>

The `FoundryOutput` that minted the token.

___

### getOutputsWithAdditionalUnlockConditions

▸ **getOutputsWithAdditionalUnlockConditions**(`outputs`): `Promise`<`string`[]\>

Get outputs with additional unlock conditions.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputs` | [`OutputsToClaim`](../enums/OutputsToClaim.md) | The type of outputs to claim. |

#### Returns

`Promise`<`string`[]\>

The output IDs of the unlockable outputs.

___

### getTransaction

▸ **getTransaction**(`transactionId`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Get a transaction stored in the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `transactionId` | `string` | The ID of the transaction to get. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The transaction.

___

### listAddresses

▸ **listAddresses**(): `Promise`<[`Address`](../interfaces/Address.md)[]\>

List all the addresses of the account.

#### Returns

`Promise`<[`Address`](../interfaces/Address.md)[]\>

The addresses.

___

### listAddressesWithUnspentOutputs

▸ **listAddressesWithUnspentOutputs**(): `Promise`<[`AddressWithUnspentOutputs`](../interfaces/AddressWithUnspentOutputs.md)[]\>

List the addresses of the account with unspent outputs.

#### Returns

`Promise`<[`AddressWithUnspentOutputs`](../interfaces/AddressWithUnspentOutputs.md)[]\>

The addresses.

___

### listOutputs

▸ **listOutputs**(`filterOptions?`): `Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

List all outputs of the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filterOptions?` | [`FilterOptions`](../interfaces/FilterOptions.md) | Options to filter the to be returned outputs. |

#### Returns

`Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

The outputs with metadata.

___

### listPendingTransactions

▸ **listPendingTransactions**(): `Promise`<[`Transaction`](../interfaces/Transaction.md)[]\>

List all the pending transactions of the account.

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)[]\>

The transactions.

___

### listIncomingTransactions

▸ **listIncomingTransactions**(): `Promise`<[`string`, `ITransactionPayload`, `IOutputResponse`][]\>

List all incoming transactions of the account.

#### Returns

`Promise`<[`string`, `ITransactionPayload`, `IOutputResponse`][]\>

The incoming transactions with their inputs.

___

### listTransactions

▸ **listTransactions**(): `Promise`<[`Transaction`](../interfaces/Transaction.md)[]\>

List all the transactions of the account.

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)[]\>

The transactions.

___

### listUnspentOutputs

▸ **listUnspentOutputs**(`filterOptions?`): `Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

List all the unspent outputs of the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filterOptions?` | [`FilterOptions`](../interfaces/FilterOptions.md) | Options to filter the to be returned outputs. |

#### Returns

`Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

The outputs with metadata.

___

### getMetadata

▸ **getMetadata**(): [`AccountMetadata`](../interfaces/AccountMetadata.md)

Get the accounts metadata.

#### Returns

[`AccountMetadata`](../interfaces/AccountMetadata.md)

The accounts metadata.

___

### minimumRequiredStorageDeposit

▸ **minimumRequiredStorageDeposit**(`output`): `Promise`<`string`\>

Calculate the minimum required storage deposit for an output.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `output` | `OutputTypes` | output to calculate the deposit amount for. |

#### Returns

`Promise`<`string`\>

The amount.

___

### increaseNativeTokenSupply

▸ **increaseNativeTokenSupply**(`tokenId`, `mintAmount`, `increaseNativeTokenSupplyOptions?`, `transactionOptions?`): `Promise`<[`MintTokenTransaction`](../interfaces/MintTokenTransaction.md)\>

Mint more native tokens.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `tokenId` | `string` | The native token id. |
| `mintAmount` | `string` | To be minted amount. |
| `increaseNativeTokenSupplyOptions?` | [`IncreaseNativeTokenSupplyOptions`](../interfaces/IncreaseNativeTokenSupplyOptions.md) | Options for minting more tokens. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`MintTokenTransaction`](../interfaces/MintTokenTransaction.md)\>

The minting transaction and the token ID.

___

### mintNativeToken

▸ **mintNativeToken**(`nativeTokenOptions`, `transactionOptions?`): `Promise`<[`MintTokenTransaction`](../interfaces/MintTokenTransaction.md)\>

Mint native tokens.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nativeTokenOptions` | [`NativeTokenOptions`](../interfaces/NativeTokenOptions.md) | The options for minting tokens. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`MintTokenTransaction`](../interfaces/MintTokenTransaction.md)\>

The minting transaction and the token ID.

___

### mintNfts

▸ **mintNfts**(`nftsOptions`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Mint nfts.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nftsOptions` | [`NftOptions`](../interfaces/NftOptions.md)[] | The options for minting nfts. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The minting transaction.

___

### prepareOutput

▸ **prepareOutput**(`options`, `transactionOptions?`): `Promise`<`OutputTypes`\>

Prepare an output for sending, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`OutputOptions`](../interfaces/OutputOptions.md) | The options for preparing an output. If the amount is below the minimum required storage deposit, by default the remaining amount will automatically be added with a `StorageDepositReturn` `UnlockCondition`, when setting the `ReturnStrategy` to `gift`, the full minimum required storage deposit will be sent  to the recipient. When the assets contain an nft id, the data from the existing `NftOutput` will be used, just with the address unlock conditions replaced. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<`OutputTypes`\>

The prepared output.

___

### prepareSendAmount

▸ **prepareSendAmount**(`addressesWithAmount`, `options?`): `Promise`<[`PreparedTransactionData`](../interfaces/PreparedTransactionData.md)\>

Prepare a send amount transaction, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesWithAmount` | [`AddressWithAmount`](../interfaces/AddressWithAmount.md)[] | Address with amounts to send. |
| `options?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`PreparedTransactionData`](../interfaces/PreparedTransactionData.md)\>

The prepared transaction data.

___

### prepareTransaction

▸ **prepareTransaction**(`outputs`, `options?`): `Promise`<[`PreparedTransactionData`](../interfaces/PreparedTransactionData.md)\>

Prepare a transaction, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputs` | `OutputTypes`[] | Outputs to use in the transaction. |
| `options?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`PreparedTransactionData`](../interfaces/PreparedTransactionData.md)\>

The prepared transaction data.

___

### sendAmount

▸ **sendAmount**(`addressesWithAmount`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Send a transaction with amounts from input addresses.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesWithAmount` | [`AddressWithAmount`](../interfaces/AddressWithAmount.md)[] | Addresses with amounts. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### sendMicroTransaction

▸ **sendMicroTransaction**(`addressesWithMicroAmount`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Send a micro transaction with amount below minimum storage deposit.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesWithMicroAmount` | [`AddressWithMicroAmount`](../interfaces/AddressWithMicroAmount.md)[] | Addresses with micro amounts. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### sendNativeTokens

▸ **sendNativeTokens**(`addressesNativeTokens`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Send native tokens.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesNativeTokens` | [`AddressNativeTokens`](../interfaces/AddressNativeTokens.md)[] | Addresses amounts and native tokens. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### sendNft

▸ **sendNft**(`addressesAndNftIds`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Send nft.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesAndNftIds` | [`AddressNftId`](../interfaces/AddressNftId.md)[] | Addresses and nft ids. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### sendOutputs

▸ **sendOutputs**(`outputs`, `transactionOptions?`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Send outputs in a transaction.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputs` | `OutputTypes`[] | The outputs to send. |
| `transactionOptions?` | [`TransactionOptions`](../interfaces/TransactionOptions.md) | The options to define a `RemainderValueStrategy` or custom inputs. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### signTransactionEssence

▸ **signTransactionEssence**(`preparedTransactionData`): `Promise`<[`SignedTransactionEssence`](../interfaces/SignedTransactionEssence.md)\>

Sign a prepared transaction, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `preparedTransactionData` | [`PreparedTransactionData`](../interfaces/PreparedTransactionData.md) | The prepared transaction data to sign. |

#### Returns

`Promise`<[`SignedTransactionEssence`](../interfaces/SignedTransactionEssence.md)\>

The signed transaction essence.

___

### submitAndStoreTransaction

▸ **submitAndStoreTransaction**(`signedTransactionData`): `Promise`<[`Transaction`](../interfaces/Transaction.md)\>

Validate the transaction, submit it to a node and store it in the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `signedTransactionData` | [`SignedTransactionEssence`](../interfaces/SignedTransactionEssence.md) | A signed transaction to submit and store. |

#### Returns

`Promise`<[`Transaction`](../interfaces/Transaction.md)\>

The sent transaction.

___

### sync

▸ **sync**(`options?`): `Promise`<[`AccountBalance`](../interfaces/AccountBalance.md)\>

Sync the account by fetching new information from the nodes.
Will also retry pending transactions if necessary.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options?` | [`AccountSyncOptions`](../interfaces/AccountSyncOptions.md) | Optional synchronization options. |

#### Returns

`Promise`<[`AccountBalance`](../interfaces/AccountBalance.md)\>

The account balance.
