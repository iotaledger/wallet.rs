# IOTA Wallet Library - Node.js binding

Node.js binding to the IOTA wallet library.

## Installation

Currently the package isn't published so you'd need to link it to your project using `npm` or `yarn`.

- Using NPM:
```
$ git clone https://github.com/iotaledger/wallet.rs
$ cd wallet.rs/bindings/node
$ npm link
$ cd /path/to/nodejs/project/
$ npm link iota-wallet
```
- Using yarn: 
```
$ git clone https://github.com/iotaledger/wallet.rs
$ cd wallet.rs/bindings/node
$ yarn link
$ cd /path/to/nodejs/project/
$ yarn link iota-wallet
```

## Getting Started

After you linked the library, you can create an `AccountManager` instance and interface with it.

```javascript
const { AccountManager } = require('iota-wallet')
const manager = new AccountManager()
const account = await manager.createAccount({
  alias: 'my first account',
  clientOptions: {
    node: 'http://localhost:14265'
  }
})
```

## API Reference

### AccountManager

#### constructor([storagePath])

Creates a new instance of the AccountManager.

| Param         | Type                | Default                | Description                                    |
| ------------- | ------------------- | ---------------------- | ---------------------------------------------- |
| [storagePath] | <code>string</code> | <code>undefined</code> | The path where the database file will be saved |

#### setStrongholdPassword(password): void

Sets the stronghold password and initialises it.

| Param    | Type                | Default                | Description                      |
| -------- | ------------------- | ---------------------- | -------------------------------- |
| password | <code>string</code> | <code>undefined</code> | The stronghold snapshot password |

#### createAccount(account): Account

Creates a new account.

| Param                 | Type                                         | Default                           | Description                                      |
| --------------------- | -------------------------------------------- | --------------------------------- | ------------------------------------------------ |
| account               | <code>object</code>                          | <code>{}</code>                   | The account to be created                        |
| account.clientOptions | <code>[ClientOptions](#clientoptions)</code> | <code>undefined</code>            | The node configuration                           |
| [account.mnemonic]    | <code>string</code>                          | <code>undefined</code>            | The account BIP39 mnemonic                       |
| [account.alias]       | <code>string</code>                          | <code>Account ${index + 1}</code> | The account alias                                |
| [account.createdAt]   | <code>string</code>                          | the current date and time         | The ISO 8601 date string of the account creation |

#### getAccount(accountId)

Gets the account with the given identifier or index.

| Param     | Type                          | Default           | Description                             |
| --------- | ----------------------------- | ----------------- | --------------------------------------- |
| accountId | <code>string \| number</code> | <code>null</code> | The account identifier or account index |

**Returns** the associated Account instance or undefined if the account wasn't found.

#### getAccountByAlias(alias)

Gets the account with the given alias (case insensitive).

| Param | Type                | Default           | Description       |
| ----- | ------------------- | ----------------- | ----------------- |
| alias | <code>string</code> | <code>null</code> | The account alias |

**Returns** the associated Account instance or undefined if the account wasn't found.

#### getAccounts()

Gets all stored accounts.

**Returns** an array of [Account objects](#account).

#### removeAccount(accountId)

Removes the account with the given identifier or index.

| Param     | Type                          | Default           | Description                             |
| --------- | ----------------------------- | ----------------- | --------------------------------------- |
| accountId | <code>string \| number</code> | <code>null</code> | The account identifier or account index |

#### syncAccounts()

Synchronize all stored accounts with the Tangle.

**Returns** A promise resolving to an array of [SyncedAccount](#syncedaccount).

#### internalTransfer(fromAccountId, toAccountId, amount)

Transfers an amount from one subaccount to another.

| Param         | Type                          | Default                | Description                                         |
| ------------- | ----------------------------- | ---------------------- | --------------------------------------------------- |
| fromAccountId | <code>string \| number</code> | <code>null</code>      | The source account identifier or account index      |
| toAccountId   | <code>string \| number</code> | <code>null</code>      | The destination account identifier or account index |
| amount        | <code>number</code>           | <code>undefined</code> | The transfer amount                                 |

**Returns** A promise resolving to the transfer's Message.

#### backup(destination)

Backups the database.

| Param       | Type                | Default                | Description                 |
| ----------- | ------------------- | ---------------------- | --------------------------- |
| destination | <code>string</code> | <code>undefined</code> | The path to the backup file |

**Returns** The full path to the backup file.

#### importAccounts(source)

Imports a database file.

| Param  | Type                | Default                | Description                 |
| ------ | ------------------- | ---------------------- | --------------------------- |
| source | <code>string</code> | <code>undefined</code> | The path to the backup file |

### SyncedAccount

#### send(address, amount)

Send funds to the given address.

| Param   | Type                | Default                | Description                               |
| ------- | ------------------- | ---------------------- | ----------------------------------------- |
| address | <code>string</code> | <code>null</code>      | The bech32 string of the transfer address |
| amount  | <code>number</code> | <code>undefined</code> | The transfer amount                       |

#### retry(messageId)

Retries (promotes or reattaches) the given message.

| Param     | Type                | Default           | Description              |
| --------- | ------------------- | ----------------- | ------------------------ |
| messageId | <code>string</code> | <code>null</code> | The message's identifier |

#### reattach(messageId)

Reattach the given message.

| Param     | Type                | Default           | Description              |
| --------- | ------------------- | ----------------- | ------------------------ |
| messageId | <code>string</code> | <code>null</code> | The message's identifier |

#### promote(messageId)

Promote the given message.

| Param     | Type                | Default           | Description              |
| --------- | ------------------- | ----------------- | ------------------------ |
| messageId | <code>string</code> | <code>null</code> | The message's identifier |


### Account

#### id()

Returns the account's identifier.

#### index()

Returns the account's index.

#### alias()

Returns the account's alias.

#### availableBalance()

Returns the account's available balance.

#### totalBalance()

Returns the account's total balance.

#### listMessages([count, from, type])

Returns the account's messages.

| Param   | Type                | Default           | Description                                                                              |
| ------- | ------------------- | ----------------- | ---------------------------------------------------------------------------------------- |
| [count] | <code>number</code> | <code>0</code>    | The number of messages to return (`0` to return all)                                     |
| [skip]  | <code>number</code> | <code>0</code>    | The number of messages to skip                                                           |
| [type]  | <code>number</code> | <code>null</code> | The message type filter (Received = 1, Sent = 2, Failed = 3, Unconfirmed = 4, Value = 5) |

Message object: { confirmed: boolean, broadcasted: boolean, incoming: boolean, value: number }

#### listAddresses([unspent])
Returns the account's addresses.

| Param     | Type                 | Default           | Description                 |
| --------- | -------------------- | ----------------- | --------------------------- |
| [unspent] | <code>boolean</code> | <code>null</code> | The `unspent` status filter |

Address object: { address: string, balance: number, keyIndex: number }

#### sync([options])

Synchronizes the account with the Tangle.

| Param                     | Type                 | Default                           | Description                            |
| ------------------------- | -------------------- | --------------------------------- | -------------------------------------- |
| [options]                 | <code>object</code>  | <code>{}</code>                   | The sync options                       |
| [options.addressIndex]    | <code>number</code>  | <code>latest address index</code> | The index of the first address to sync |
| [options.gapLimit]        | <code>number</code>  | <code>10</code>                   | The number of addresses to check       |
| [options.skipPersistance] | <code>boolean</code> | <code>false</code>                | Skip updating the account in storage   |

**Returns** a [SyncedAccount](#syncedaccount) instance.

#### setAlias(alias)

Updates the account alias.

| Param | Type                | Default           | Description           |
| ----- | ------------------- | ----------------- | --------------------- |
| alias | <code>string</code> | <code>null</code> | The new account alias |

#### setClientOptions(options)

Updates the account client options.

| Param   | Type                                         | Default           | Description                    |
| ------- | -------------------------------------------- | ----------------- | ------------------------------ |
| options | <code>[ClientOptions](#clientoptions)</code> | <code>null</code> | The new account client options |

#### getMessage(messageId)

Gets the message associated with the given identifier.

| Param     | Type                | Default           | Description              |
| --------- | ------------------- | ----------------- | ------------------------ |
| messageId | <code>string</code> | <code>null</code> | The message's identifier |

#### generateAddress()

Generates a new unused address and returns it.

#### latestAddress()

Returns the latest address (the one with the biggest keyIndex) or undefined if the account address list is empty.

### ClientOptions

| Field             | Type                  | Default                | Description                                                                                              |
| ----------------- | --------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------- |
| [network]         | <code>number</code>   | <code>undefined</code> | The tangle network to connect to (Mainnet = 1, Devnet = 1, Comnet = 3)                                   |
| [node]            | <code>string</code>   | <code>undefined</code> | A node URL to connect to                                                                                 |
| [nodes]           | <code>string[]</code> | <code>undefined</code> | A list node URL to connect to                                                                            |
| [quorumSize]      | <code>number</code>   | <code>undefined</code> | If multiple nodes are provided, quorum size determines the number of nodes to query to check for quorum. |
| [quorumThreshold] | <code>number</code>   | <code>undefined</code> | Minimum number of nodes from the quorum pool that need to agree to consider a result true.               |
