# Class: Account

The Account class.

## Table of contents

### Methods

- [buildAliasOutput](Account.md#buildaliasoutput)
- [buildBasicOutput](Account.md#buildbasicoutput)
- [buildFoundryOutput](Account.md#buildfoundryoutput)
- [buildNftOutput](Account.md#buildnftoutput)
- [claimOutputs](Account.md#claimoutputs)
- [consolidateOutputs](Account.md#consolidateoutputs)
- [generateAddress](Account.md#generateaddress)
- [generateAddresses](Account.md#generateaddresses)
- [getAlias](Account.md#getalias)
- [getBalance](Account.md#getbalance)
- [getOutput](Account.md#getoutput)
- [getFoundryOutput](Account.md#getfoundryoutput)
- [getOutputsWithAdditionalUnlockConditions](Account.md#getoutputswithadditionalunlockconditions)
- [getTransaction](Account.md#gettransaction)
- [listAddresses](Account.md#listaddresses)
- [listAddressesWithUnspentOutputs](Account.md#listaddresseswithunspentoutputs)
- [listOutputs](Account.md#listoutputs)
- [listPendingTransactions](Account.md#listpendingtransactions)
- [listTransactions](Account.md#listtransactions)
- [listUnspentOutputs](Account.md#listunspentoutputs)
- [minimumRequiredStorageDeposit](Account.md#minimumrequiredstoragedeposit)
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
- [tryClaimOutputs](Account.md#tryclaimoutputs)

## Methods

### buildAliasOutput

▸ **buildAliasOutput**(`data`): `Promise`<`IAliasOutput`\>

Build an `AliasOutput`.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `data` | `BuildAliasOutputData` | Options for building an `AliasOutput`. |

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
| `data` | `BuildBasicOutputData` | Options for building a `BasicOutput`. |

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
| `data` | `BuildFoundryOutputData` | Options for building a `FoundryOutput`. |

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
| `data` | `BuildNftOutputData` | Options for building an `NftOutput`. |

#### Returns

`Promise`<`INftOutput`\>

The built `NftOutput`.

___

### claimOutputs

▸ **claimOutputs**(`outputIds`): `Promise`<`Transaction`[]\>

Claim basic or nft outputs that have additional unlock conditions
to their `AddressUnlockCondition` from the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputIds` | `string`[] | The outputs to claim. |

#### Returns

`Promise`<`Transaction`[]\>

The resulting transactions.

___

### consolidateOutputs

▸ **consolidateOutputs**(`force`, `outputConsolidationThreshold?`): `Promise`<`Transaction`[]\>

Consolidate basic outputs with only an `AddressUnlockCondition` from an account
by sending them to the same address again if the output amount is greater or
equal to the output consolidation threshold.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `force` | `boolean` | Force consolidation on addresses where the threshold isn't met. |
| `outputConsolidationThreshold?` | `number` | A default threshold is used if this is omitted. |

#### Returns

`Promise`<`Transaction`[]\>

The consolidation transactions.

___

### generateAddress

▸ **generateAddress**(`options?`): `Promise`<`Address`\>

Generate a new unused address.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options?` | `AddressGenerationOptions` | Options for address generation. |

#### Returns

`Promise`<`Address`\>

The address.

___

### generateAddresses

▸ **generateAddresses**(`amount`, `options?`): `Promise`<`Address`[]\>

Generate new unused addresses.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `amount` | `number` | The amount of addresses to generate. |
| `options?` | `AddressGenerationOptions` | Options for address generation. |

#### Returns

`Promise`<`Address`[]\>

The addresses.

___

### getAlias

▸ **getAlias**(): `string`

Get this accounts alias.

#### Returns

`string`

The accounts alias.

___

### getBalance

▸ **getBalance**(): `Promise`<`AccountBalance`\>

Get the account balance.

#### Returns

`Promise`<`AccountBalance`\>

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
| `outputs` | `OutputsToClaim` | The type of outputs to claim. |

#### Returns

`Promise`<`string`[]\>

The output IDs of the unlockable outputs.

___

### getTransaction

▸ **getTransaction**(`transactionId`): `Promise`<`Transaction`\>

Get a transaction stored in the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `transactionId` | `string` | The ID of the transaction to get. |

#### Returns

`Promise`<`Transaction`\>

The transaction.

___

### listAddresses

▸ **listAddresses**(): `Promise`<`Address`[]\>

List all the addresses of the account.

#### Returns

`Promise`<`Address`[]\>

The addresses.

___

### listAddressesWithUnspentOutputs

▸ **listAddressesWithUnspentOutputs**(): `Promise`<`AddressWithUnspentOutputs`[]\>

List the addresses of the account with unspent outputs.

#### Returns

`Promise`<`AddressWithUnspentOutputs`[]\>

The addresses.

___

### listOutputs

▸ **listOutputs**(): `Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

List all outputs of the account.

#### Returns

`Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

The outputs with metadata.

___

### listPendingTransactions

▸ **listPendingTransactions**(): `Promise`<`Transaction`[]\>

List all the pending transactions of the account.

#### Returns

`Promise`<`Transaction`[]\>

The transactions.

___

### listTransactions

▸ **listTransactions**(): `Promise`<`Transaction`[]\>

List all the transactions of the account.

#### Returns

`Promise`<`Transaction`[]\>

The transactions.

___

### listUnspentOutputs

▸ **listUnspentOutputs**(): `Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

List all the unspent outputs of the account.

#### Returns

`Promise`<[`OutputData`](../interfaces/OutputData.md)[]\>

The outputs with metadata.

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

### mintNativeToken

▸ **mintNativeToken**(`nativeTokenOptions`, `transactionOptions?`): `Promise`<`MintTokenTransaction`\>

Mint native tokens.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nativeTokenOptions` | `NativeTokenOptions` | The options for minting tokens. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`MintTokenTransaction`\>

The minting transaction and the token ID.

___

### mintNfts

▸ **mintNfts**(`nftsOptions`, `transactionOptions?`): `Promise`<`Transaction`\>

Mint nfts.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nftsOptions` | `NftOptions`[] | The options for minting nfts. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The minting transaction.

___

### prepareOutput

▸ **prepareOutput**(`options`, `transactionOptions?`): `Promise`<`OutputTypes`\>

Prepare an output for sending, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | `OutputOptions` | The options for preparing an output. If the amount is  below the minimum required storage deposit, by default the remaining  amount will automatically be added with a `StorageDepositReturn` `UnlockCondition`,  when setting the `ReturnStrategy` to `gift`, the full minimum required  storage deposit will be sent  to the recipient. When the assets contain  an nft id, the data from the exisiting `NftOutput` will be used, just with  the address unlock conditions replaced. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

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
| `addressesWithAmount` | `AddressWithAmount`[] | Address with amounts to send. |
| `options?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

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
| `options?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<[`PreparedTransactionData`](../interfaces/PreparedTransactionData.md)\>

The prepared transaction data.

___

### sendAmount

▸ **sendAmount**(`addressesWithAmount`, `transactionOptions?`): `Promise`<`Transaction`\>

Send a transaction with amounts from input addresses.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesWithAmount` | `AddressWithAmount`[] | Addresses with amounts. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### sendMicroTransaction

▸ **sendMicroTransaction**(`addressesWithMicroAmount`, `transactionOptions?`): `Promise`<`Transaction`\>

Send a micro transaction with amount below minimum storage deposit.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesWithMicroAmount` | `AddressWithMicroAmount`[] | Addresses with micro amounts. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### sendNativeTokens

▸ **sendNativeTokens**(`addressesNativeTokens`, `transactionOptions?`): `Promise`<`Transaction`\>

Send native tokens.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesNativeTokens` | `AddressNativeTokens`[] | Addresses amounts and native tokens. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### sendNft

▸ **sendNft**(`addressesAndNftIds`, `transactionOptions?`): `Promise`<`Transaction`\>

Send nft.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `addressesAndNftIds` | `AddressNftId`[] | Addresses and nft ids. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### sendOutputs

▸ **sendOutputs**(`outputs`, `transactionOptions?`): `Promise`<`Transaction`\>

Send outputs in a transaction.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputs` | `OutputTypes`[] | The outputs to send. |
| `transactionOptions?` | `TransactionOptions` | The options to define a `RemainderValueStrategy`  or custom inputs. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### signTransactionEssence

▸ **signTransactionEssence**(`preparedTransactionData`): `Promise`<`SignedTransactionEssence`\>

Sign a prepared transaction, useful for offline signing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `preparedTransactionData` | [`PreparedTransactionData`](../interfaces/PreparedTransactionData.md) | The prepared transaction data to sign. |

#### Returns

`Promise`<`SignedTransactionEssence`\>

The signed transaction essence.

___

### submitAndStoreTransaction

▸ **submitAndStoreTransaction**(`signedTransactionData`): `Promise`<`Transaction`\>

Validate the transaction, submit it to a node and store it in the account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `signedTransactionData` | `SignedTransactionEssence` | A signed transaction to submit and store. |

#### Returns

`Promise`<`Transaction`\>

The sent transaction.

___

### sync

▸ **sync**(`options?`): `Promise`<`AccountBalance`\>

Sync the account by fetching new information from the nodes.
Will also retry pending transactions if necessary.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options?` | `AccountSyncOptions` | Optional synchronization options. |

#### Returns

`Promise`<`AccountBalance`\>

The account balance.

___

### tryClaimOutputs

▸ **tryClaimOutputs**(`outputsToClaim`): `Promise`<`Transaction`[]\>

Try to claim basic outputs that have additional unlock conditions to
their `AddressUnlockCondition` and send them to the first address of the
account.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `outputsToClaim` | `OutputsToClaim` | Outputs to try to claim. |

#### Returns

`Promise`<`Transaction`[]\>

The resulting transactions.
