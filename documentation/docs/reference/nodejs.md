---
description: Official IOTA Wallet Library Software Node.js API reference.
image: /img/logo/logo_dark.svg
keywords:
- api
- nodejs
- param
- type
- reference
---
# Node.js API Reference

## initLogger(config: LogOptions)

Initializes the logging system.

### LogOptions

| Param         | Type          | Default     | Description                             |
| ------------- | ------------- | ----------- | --------------------------------------- |
| color_enabled | `boolean`     | `undefined` | Whether to enable colored output or not |
| outputs       | `LogOutput[]` | `undefined` | The log outputs                         |

### LogOutput

| Param          | Type       | Default     | Description                                          |
| -------------- | ---------- | ----------- | ---------------------------------------------------- |
| name           | `string`   | `undefined` | 'stdout' or a path to a file                         |
| level_filter   | `string`   | `'info'`    | The maximum log level that this output accepts       |
| target_filters | `string[]` | `[]`        | Filters on the log target (library and module names) |

## addEventListener(event, cb)

Adds a new event listener with a callback in the form of `(err, data) => {}`.

Supported event names:
- ErrorThrown
- BalanceChange
- NewTransaction
- ConfirmationStateChange
- Reattachment
- Broadcast
- TransferProgress
- MigrationProgress

## AccountManager

### constructor([ManagerOptions])

Creates a new instance of the AccountManager.

| Param                             | Type     | Default     | Description                                  |
| --------------------------------- | -------- | ----------- | -------------------------------------------- |
| [ManagerOptions](#manageroptions) | `object` | `undefined` | The options to configure the account manager |

#### ManagerOptions 
You can use any of the following parameters when constructing the ManagerOptions. All the parameters are optional.   

| Param                            | Type      | Default     | Description                                                                               |
| -------------------------------- | --------- | ----------- | ----------------------------------------------------------------------------------------- |
| storagePath                      | `string`  | `undefined` | The path where the database file will be saved                                            |
| storagePassword                  | `string`  | `undefined` | The storage password                                                                      |
| outputConsolidationThreshold     | `number`  | `100`       | The number of outputs an address must have to trigger the automatic consolidation process |
| automaticOutputConsolidation     | `boolean` | `true`      | Disables the automatic output consolidation if false                                      |
| syncSpentOutputs                 | `boolean` | `false`     | Enables fetching spent output history on account sync                                     |
| persistEvents                    | `boolean` | `false`     | Enables event persistence                                                                 |
| allowCreateMultipleEmptyAccounts | `boolean` | `false`     | Enables creating accounts with latest account being empty                                 |
| skipPolling                      | `boolean` | `false`     | Enables creating accounts without automatic polling (background syncing)                  |
| pollingInterval                  | `number`  | `30`        | Sets the polling interval in seconds                                                      |

### setStrongholdPassword(password): void

Sets the Stronghold password and initialises it.

| Param    | Type     | Default     | Description                      |
| -------- | -------- | ----------- | -------------------------------- |
| password | `string` | `undefined` | The Stronghold snapshot password |

### changeStrongholdPassword(currentPassword, newPassword): void

Changes the Stronghold password.

| Param           | Type     | Default     | Description                     |
| --------------- | -------- | ----------- | ------------------------------- |
| currentPassword | `string` | `undefined` | The current Stronghold password |
| newPassword     | `string` | `undefined` | The new Stronghold password     |

### createAccount(account): Account

Creates a new account.

| Param                 | Type                              | Default                   | Description                                              |
| --------------------- | --------------------------------- | ------------------------- | -------------------------------------------------------- |
| account               | `object`                          | `{}`                      | The account to be created                                |
| account.clientOptions | `[ClientOptions](#clientoptions)` | `undefined`               | The node configuration                                   |
| [account.mnemonic]    | `string`                          | `undefined`               | The account BIP39 mnemonic                               |
| [account.alias]       | `string`                          | `Account ${index + 1}`    | The account alias                                        |
| [account.createdAt]   | `string`                          | the current date and time | The ISO 8601 date string of the account creation         |
| [account.signerType]  | `number`                          | 1 = Stronghold            | The account signer type. 1 = Stronghold, 2 = EnvMnemonic |

### getAccount(accountId)

Gets the account with the given identifier or index.

| Param     | Type               | Default | Description                             |
| --------- | ------------------ | ------- | --------------------------------------- |
| accountId | `string \| number` | `null`  | The account identifier or account index |

Returns the associated Account instance or undefined if the account wasn't found.

### getAccountByAlias(alias)

Gets the account with the given alias (case-insensitive).

| Param | Type     | Default | Description       |
| ----- | -------- | ------- | ----------------- |
| alias | `string` | `null`  | The account alias |

Returns the associated Account instance or undefined if the account wasn't found.

### getAccounts()

Gets all stored accounts.

Returns an array of [Account objects](#account).

### removeAccount(accountId)

Removes the account with the given identifier or index.

| Param     | Type               | Default | Description                             |
| --------- | ------------------ | ------- | --------------------------------------- |
| accountId | `string \| number` | `null`  | The account identifier or account index |


### startBackgroundSync(pollingInterval, automaticOutputConsolidation): Promise<void/>

Starts the background polling and MQTT monitoring.

| Param                        | Type      | Default | Description                                      |
| ---------------------------- | --------- | ------- | ------------------------------------------------ |
| pollingInterval              | `number`  | `null`  | The polling interval in seconds                  |
| automaticOutputConsolidation | `boolean` | `null`  | If outputs should get consolidated automatically |

### stop_background_sync(): void

Stops the background polling and MQTT monitoring.

### syncAccounts([options])

Synchronize all stored accounts with the Tangle.

| Param                  | Type     | Default                | Description                                           |
| ---------------------- | -------- | ---------------------- | ----------------------------------------------------- |
| [options]              | `object` | `{}`                   | The sync options                                      |
| [options.addressIndex] | `number` | `latest address index` | The index of the first account address to sync        |
| [options.gapLimit]     | `number` | `10`                   | The number of addresses to check on each account sync |

Returns a promise resolving to an array of [SyncedAccount](#syncedaccount).

### internalTransfer(fromAccount, toAccount, amount)

Transfers an amount from one sub-account to another.

| Param       | Type                  | Default     | Description             |
| ----------- | --------------------- | ----------- | ----------------------- |
| fromAccount | `[Account](#account)` | `null`      | The source account      |
| toAccount   | `[Account](#account)` | `null`      | The destination account |
| amount      | `number`              | `undefined` | The transfer amount     |

Returns a promise resolving to the transfer's Message.

### backup(destination, password)

Backups the database.

| Param       | Type     | Default     | Description                    |
| ----------- | -------- | ----------- | ------------------------------ |
| destination | `string` | `undefined` | The path to the backup file    |
| password    | `string` | `undefined` | The backup Stronghold password |

Returns the full path to the backup file.

### importAccounts(source)

Imports a database file.

| Param    | Type     | Default     | Description                    |
| -------- | -------- | ----------- | ------------------------------ |
| source   | `string` | `undefined` | The path to the backup file    |
| password | `string` | `undefined` | The backup Stronghold password |

### isLatestAddressUnused()

Determines whether all accounts have unused their latest address after syncing with the Tangle.

Returns a promise resolving to the boolean value.

### setClientOptions(options)

Updates the client options for all accounts.

| Param   | Type                              | Default | Description                    |
| ------- | --------------------------------- | ------- | ------------------------------ |
| options | `[ClientOptions](#clientoptions)` | `null`  | The new account client options |

### generateMigrationAddress(address)

Convert a Ed25519 to a Tryte migration address with checksum (last 9 Trytes)

| Param   | Type     | Default | Description                    |
| ------- | -------- | ------- | ------------------------------ |
| address | `string` | `null`  | Bech32 encoded Ed25519 address |

### getBalanceChangeEvents([count, skip, fromTimestamp])

Gets the persisted balance change events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [count]         | `number` | `0`     | The number of events to return (`0` to return all)           |
| [skip]          | `number` | `0`     | The number of events to skip                                 |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { indexationId: string, accountId: string, messageId?: string, remainder?: boolean, balanceChange: { spent: number, received: number } }

### getBalanceChangeEventCount([fromTimestamp])

Gets the number of persisted balance change events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### getTransactionConfirmationEvents([count, skip, fromTimestamp])

Gets the persisted transaction confirmation change events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [count]         | `number` | `0`     | The number of events to return (`0` to return all)           |
| [skip]          | `number` | `0`     | The number of events to skip                                 |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { indexationId: string, accountId: string, message: Message, confirmed: boolean }

### getTransactionConfirmationEventCount([fromTimestamp])

Gets the number of persisted transaction confirmation change events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### getNewTransactionEvents([count, skip, fromTimestamp])

Gets the persisted new transaction events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [count]         | `number` | `0`     | The number of events to return (`0` to return all)           |
| [skip]          | `number` | `0`     | The number of events to skip                                 |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { indexationId: string, accountId: string, message: Message }

### getNewTransactionEventCount([fromTimestamp])

Gets the number of persisted new transaction events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### getReattachmentEvents([count, skip, fromTimestamp])

Gets the persisted transaction reattachment events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [count]         | `number` | `0`     | The number of events to return (`0` to return all)           |
| [skip]          | `number` | `0`     | The number of events to skip                                 |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { indexationId: string, accountId: string, message: Message }

### getReattachmentEventCount([fromTimestamp])

Gets the number of persisted transaction reattachment events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### getBroadcastEvents([count, skip, fromTimestamp])

Gets the persisted transaction broadcast events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [count]         | `number` | `0`     | The number of events to return (`0` to return all)           |
| [skip]          | `number` | `0`     | The number of events to skip                                 |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { indexationId: string, accountId: string, message: Message }

### getBroadcastEventCount([fromTimestamp])

Gets the number of persisted transaction broadcast events.

| Param           | Type     | Default | Description                                                  |
| --------------- | -------- | ------- | ------------------------------------------------------------ |
| [fromTimestamp] | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

## SyncedAccount

The result of a `sync` operation on an Account.

## Account

### id()

Returns the account's identifier.

### index()

Returns the account's index.

### alias()

Returns the account's alias.

### balance(): AccountBalance

Returns the account's balance information object.

Balance object: { total: number, available: number, incoming: number, outgoing: number }

### messageCount([type])

Returns the number of messages associated with the account.

| Param  | Type     | Default | Description                                                                              |
| ------ | -------- | ------- | ---------------------------------------------------------------------------------------- |
| [type] | `number` | `null`  | The message type filter (Received = 1, Sent = 2, Failed = 3, Unconfirmed = 4, Value = 5) |

### listMessages([count, from, type])

Returns the account's messages.

| Param   | Type     | Default | Description                                                                              |
| ------- | -------- | ------- | ---------------------------------------------------------------------------------------- |
| [count] | `number` | `0`     | The number of messages to return (`0` to return all)                                     |
| [skip]  | `number` | `0`     | The number of messages to skip                                                           |
| [type]  | `number` | `null`  | The message type filter (Received = 1, Sent = 2, Failed = 3, Unconfirmed = 4, Value = 5) |

Message object: { confirmed: boolean, broadcasted: boolean, incoming: boolean, value: number }

### listAddresses([unspent])
Returns the account's addresses.

| Param     | Type      | Default | Description               |
| --------- | --------- | ------- | ------------------------- |
| [unspent] | `boolean` | `null`  | The unspent status filter |

Address object: { address: string, keyIndex: number }

### sync([options])

Synchronizes the account with the Tangle.

| Param                  | Type     | Default                | Description                            |
| ---------------------- | -------- | ---------------------- | -------------------------------------- |
| [options]              | `object` | `{}`                   | The sync options                       |
| [options.addressIndex] | `number` | `latest address index` | The index of the first address to sync |
| [options.gapLimit]     | `number` | `10`                   | The number of addresses to check       |

Returns a [SyncedAccount](#syncedaccount) instance.

### send(address, amount[, options])

Send funds to the given address.

| Param   | Type              | Default     | Description                               |
| ------- | ----------------- | ----------- | ----------------------------------------- |
| address | `string`          | `null`      | The bech32 string of the transfer address |
| amount  | `number`          | `undefined` | The transfer amount                       |
| options | `TransferOptions` | `undefined` | The transfer options                      |

#### TransferOptions

| Param                  | Type                                   | Default                 | Description                                           |
| ---------------------- | -------------------------------------- | ----------------------- | ----------------------------------------------------- |
| remainderValueStrategy | `RemainderValueStrategy`               | `null`                  | The strategy to use for the remainder value if any    |
| indexation             | `{ index: string, data?: Uint8Array }` | `null`                  | Message indexation                                    |
| skipSync               | `boolean`                              | `false`                 | Send transfer without synchronising the account first |
| outputKind             | `OutputKind`                           | `signatureLockedSingle` | Output kind                                           |

#### RemainderValueStrategy

##### changeAddress()
Send the remainder value to an internal address.

###### reuseAddress()
Send the remainder value to its original address.

###### accountAddress(address: string)
Send the remainder value to a specific address that must belong to the account.

### getNodeInfo([url, auth]): NodeInfoWrapper

Gets information about the node.

| Param | Type     | Default                    | Description           |
| ----- | -------- | -------------------------- | --------------------- |
| url   | `string` | `Node from client options` | The node url          |
| auth  | `Auth`   | `undefined`                | The node auth options |

Returns the [NodeInfoWrapper](#nodeinfowrapper)

#### NodeInfoWrapper

| Param    | Type                  | Default                    | Description   |
| -------- | --------------------- | -------------------------- | ------------- |
| url      | `string`              | `Node from client options` | The node url  |
| nodeinfo | [NodeInfo](#nodeinfo) | `null`                     | The node info |

##### NodeInfo

All the values are for the NodeInfo are set by the nodes.

| Param                         | Type       | Default| Description                                  |
| ----------------------------- | ---------- | ------ | -------------------------------------------- |
| [name]                        | `string`   | `null` | The node name                                |
| [version]                     | `string`   | `null` | The node version                             |
| [isHealthy]                   | `boolean`  | `null` | Indicates if the node is healthy             |
| [networkId]                   | `number`   | `null` | The network ID                               |
| [bech32HRP]                   | `string`   | `null` | The human-readable part of the bech32 string |
| [minPoWScore]                 | `number`   | `null` | The node minimum proof of work score         |
| [messagesPerSecond]           | `number`   | `null` | The node messages per second                 |
| [referencedMessagesPerSecond] | `number`   | `null` | The node references per second               |
| [referencedRate]              | `number`   | `null` | The node reference rate                      |
| [latestMilestoneTimestamp]    | `number`   | `null` | The node's latest milestone timestamp        |
| [latestMilestoneIndex]        | `number`   | `null` | The node's latest milestone index            |
| [confirmedMilestoneIndex]     | `number`   | `null` | The node's confirmed milestone index         |
| [pruningIndex]                | `number`   | `null` | The node's pruning index                     |
| [features]                    | `string[]` | `null` | The node's features.                         |

### retry(messageId)

Retries (promotes or reattaches) the given message.

| Param     | Type     | Default | Description              |
| --------- | -------- | ------- | ------------------------ |
| messageId | `string` | `null`  | The message's identifier |

### reattach(messageId)

Reattach the given message.

| Param     | Type     | Default | Description              |
| --------- | -------- | ------- | ------------------------ |
| messageId | `string` | `null`  | The message's identifier |

### promote(messageId)

Promote the given message.

| Param     | Type     | Default | Description              |
| --------- | -------- | ------- | ------------------------ |
| messageId | `string` | `null`  | The message's identifier |

### consolidateOutputs([includeDustAllowanceOutputs])

Consolidate the outputs on all account addresses.

| Param                       | Type      | Default | Description                                       |
| --------------------------- | --------- | ------- | ------------------------------------------------- |
| includeDustAllowanceOutputs | `boolean` | `false` | If true, also consolidates dust allowance outputs |

### isLatestAddressUnused()

Determines whether the account has an unused latest address after syncing with the Tangle.

Returns a promise resolving to the boolean value.

### setAlias(alias)

Updates the account alias.

| Param | Type     | Default | Description           |
| ----- | -------- | ------- | --------------------- |
| alias | `string` | `null`  | The new account alias |

### setClientOptions(options)

Updates the account client options.

| Param   | Type                              | Default | Description                    |
| ------- | --------------------------------- | ------- | ------------------------------ |
| options | `[ClientOptions](#clientoptions)` | `null`  | The new account client options |

### getMessage(messageId)

Gets the message associated with the given identifier.

| Param     | Type     | Default | Description              |
| --------- | -------- | ------- | ------------------------ |
| messageId | `string` | `null`  | The message's identifier |

### getAddress(addressBech32)

Gets the address object by its bech32 representation.

| Param         | Type     | Default | Description                       |
| ------------- | -------- | ------- | --------------------------------- |
| addressBech32 | `string` | `null`  | The address bech32 representation |

### generateAddress()

Generates a new unused address and returns it.

| Param  | Type     | Default     | Description             |
| ------ | -------- | ----------- | ----------------------- |
| amount | `number` | `undefined` | The amount of addresses |

### latestAddress()

Returns the latest address (the one with the biggest keyIndex).

### getUnusedAddress()

Synchronizes the account addresses with the Tangle and returns the latest address in the account,
which is an address without balance.

## ClientOptions

| Field               | Type                        | Default     | Description                                                                                              |
| ------------------- | --------------------------- | ----------- | -------------------------------------------------------------------------------------------------------- |
| [network]           | `number`                    | `undefined` | The tangle network to connect to (Mainnet = 1, Devnet = 1, Comnet = 3)                                   |
| [primaryNode]       | `NodeUrl | [Node](#node)`   | `undefined` | A node URL to alway connect to first                                                                     |
| [primaryPoWNode]    | `NodeUrl | [Node](#node)`   | `undefined` | A node URL to alway connect to first when using remote PoW, will be used before primaryNode              |
| [node]              | `NodeUrl | [Node](#node)`   | `undefined` | A node URL to connect to                                                                                 |
| [nodes]             | `NodeUrl | [Node](#node)[]` | `undefined` | A list node URL to connect to                                                                            |
| [quorumSize]        | `number`                    | `undefined` | If multiple nodes are provided, quorum size determines the number of nodes to query to check for quorum. |
| [quorumThreshold]   | `number`                    | `undefined` | Minimum number of nodes from the quorum pool that need to agree to consider a result true.               |
| [localPow]          | `boolean`                   | `true`      | Whether to use local or remote PoW.                                                                      |
| [MqttBrokerOptions] | `MqttBrokerOptions`         | `undefined` | Options for the MQTT broker                                                                              |

## MqttBrokerOptions

All fields are optional.

| Field                   | Type      | Description                                                                                           |
| ----------------------- | --------- | ----------------------------------------------------------------------------------------------------- |
| automaticDisconnect     | `boolean` | Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not. |
| timeout                 | `number`  | MQTT connection timeout in seconds                                                                    |
| useWs                   | `boolean` | Defines if websockets should be used (true) or TCP (false)                                            |
| maxReconnectionAttempts | `number`  | Defines the maximum reconnection attempts before it returns an error                                  |
| port                    | `number`  | Defines the port to be used for the MQTT connection                                                   |

### Auth
| Field      | Type     | Default     | Description                                |
| ---------- | -------- | ----------- | ------------------------------------------ |
| [jwt]      | `string` | `undefined` | Optional JSON Web Token.                   |
| [username] | `string` | `undefined` | Optional name for basic authentication     |
| [password] | `string` | `undefined` | Optional password for basic authentication |

### Node

NodeUrl = string

| Field      | Type      | Default     | Description                                |
| ---------- | --------- | ----------- | ------------------------------------------ |
| [url]      | `NodeUrl` | `undefined` | Node url                                   |
| [auth]     | `Auth`    | `undefined` | Optional authentication options            |
| [disabled] | `boolean` | `false`     | Optional password for basic authentication |

### OutputKind

Possible output kinds.

| Field                          | Type       | Default | Description                                        |
| ------------------------------ | ---------- | ------- | -------------------------------------------------- |
| [signatureLockedSingle]        | `string`   | `null`  | Default output type                                |
| [signatureLockedDustAllowance] | `string`   | `null`  | Output type to enable receiving dust on an address |
