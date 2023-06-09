---
description: The wallet.rs library is a stateful package with a standardized interface to build applications with IOTA value transactions.
image: /img/logo/logo.svg
keywords:
- wallet library methods
- Rust
- messages
- Required client library
- access modifiers
- types
- reference
---
# Wallet Library Specifications

## Introduction

The `wallet.rs` library is a stateful package with a standardized interface to build applications with IOTA value transactions. The package is compatible with different platforms such as web, desktop, and mobile. 

The package introduces the concept of an _account_ . An _account_ is a reference to, or a label for, a [seed](https://wiki.iota.org/introduction/reference/details/#seed). It has certain properties such as [addresses](https://github.com/Wollac/protocol-rfcs/blob/bech32-address-format/text/0020-bech32-address-format/0020-bech32-address-format.md) and [messages](https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md). An account also maintains various behaviours, including moving funds, looking for new messages, and making copies of message histories. Additionally, it provides a degree of financial privacy and thus does not incur any overhead. 

[A similar account package was used before](https://wiki.iota.org/introduction/explanations/update/chrysalis_improvements/#ed25519-signature-scheme) but it became obsolete with the introduction of Ed25519 signatures. The previous account package was also limited to a single account, whereas the new package manages multiple accounts. 

For IOTA, the motivation to use this package was to offer a simplified (stateful) approach to handle IOTA payments.

## Considerations

*   Seeds should be stored and managed separately in a secure enclave and should never leave the secure environment. Secure enclaves include software enclaves such as IOTA’s Rust-based `Stronghold` library or hardware enclaves such as a `Ledger Nano`.

*   The secure enclave should have the ability to generate addresses and sign messages upon receipt, and return the output in a new message. If the secure enclave is initialized with a pre-generated seed, the sender process should immediately remove the seed traces from memory. 

## Naming Conventions

The primary language is [Rust](https://github.com/rust-lang/rust). Therefore, you should follow the standard [Rust naming conventions](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html). For reference, all interfaces (types) use _CamelCase_ while all function and variable names use _snake\_case_.

## Interfaces

#### AccountConfiguration

Account configuration or initialization object. It should support parameters accepted by high level [client](https://github.com/iotaledger/iota.rs) libraries.

| Property         | Required | Type                                                                                                             | Description                                                                                                                                                            |
| ---------------- | -------- | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| seed             | ✘        | string                                                                                                           | BIP-39 mnemonic. When importing an account from Stronghold backup, the seed will not be required.                                                                      |
| id               | ✘        | string                                                                                                           | SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in Stronghold. The ID should be provided by Stronghold.             |
| index            | ✔        | number                                                                                                           | Account index in [BIP-44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki) derivation path                                                              |
| alias            | ✘        | string                                                                                                           | Account name. If not provided, `Account + { index }` should be used. When importing an account from Stronghold backup, the alias will be required from Stronghold. |
| pow              | ✘        | `local`,`remote`                                                                                                 | Proof of work settings. Defaults to `local`.  `local` PoW  should be performed on device; `remote` PoW should be performed on the node.               |
| nodes            | ✔        | [node](#node)[]                                                                                                  | A list of nodes to connect to.                                                                                                                                         |
| quorum_size      | ✘        | number                                                                                                           | If multiple nodes are provided, quorum size determines the number of nodes to query to check for quorum.                                                               |
| quorum_threshold | ✘        | number                                                                                                           | Minimum number of nodes from the quorum pool that need to agree to consider a result true.                                                                             |
| network          | ✘        | `mainnet`\|`devnet` \| `comnet`                                                                                  | IOTA public network.                                                                                                                                                   |
| type             | ✘        | `default` or `ledger`                                                                                            | Account type. Required for differentiating ledger vs non-ledger accounts.                                                                                              |
| provider         | ✘        | string                                                                                                           | Node URL.                                                                                                                                                              |
| created_at       | ✘        | Date                                                                                                             | Time of account creation                                                                                                                                               |
| messages         | ✘        | [Message](#message)[] Messages associated with account. Accounts can be initialized with locally stored messages. |
| addresses        | ✘        | [Address](#address)[]                                                                                            | Address history associated with the account. Accounts can be initialized with locally stored address history                                                           |

#### AccountObject

| Property                    | Required | Type     | Description                                                                                                     |
| --------------------------- | -------- | -------- | --------------------------------------------------------------------------------------------------------------- |
| id                          | ✔        | string   | SHA-256 hash of the first address on the seed (m/44'/0'/0'/0/0). Required for referencing a seed in Stronghold. |
| alias                       | ✔        | string   | Account name.                                                                                                   |
| created_at                  | ✔        | number   | Account creation time.                                                                                          |
| last_synced_at              | ✔        | string   | Time the account was last synced with the Tangle.                                                               |
| sync()                      | ✔        | function | Syncs account with the Tangle.                                                                                  |
| reattach()                  | ✔        | function | Reattaches unconfirmed transaction to the Tangle.                                                               |
| total_balance()             | ✔        | function | Gets total account balance.                                                                                     |
| available_balance()         | ✔        | function | Gets available account balance.                                                                                 |
| set_alias()                 | ✔        | function | Updates account name.                                                                                           |
| list_messages()             | ✔        | function | Gets messages.                                                                                                  |
| list_received_messages()    | ✔        | function | Gets all received messages.                                                                                     |
| list_sent_messages()        | ✔        | function | Gets all sent messages.                                                                                         |
| list_failed_messages()      | ✔        | function | Gets all failed messages.                                                                                       |
| list_unconfirmed_messages() | ✔        | function | Gets all unconfirmed messages.                                                                                  |
| get_message()               | ✔        | function | Gets message for providedID.                                                                                    |
| list_addresses()            | ✔        | function | Gets all addresses.                                                                                             |
| list_unspent_addresses()    | ✔        | function | Gets all unspent input addresses.                                                                               |
| generate_address()          | ✔        | function | Gets the latest unused address.                                                                                 |

#### SyncedAccountObject

| Property        | Required | Type     | Description                                                                                        |
| --------------- | -------- | -------- | -------------------------------------------------------------------------------------------------- |
| deposit_address | ✔        | Address  | Deposit address. Only exposed on successful completion of account syncing process.                 |
| send()          | ✔        | function | Send transaction method. Only exposed on successful completion of account syncing process.         |
| retry()         | ✔        | function | Rebroadcasts failed transaction. Only exposed on successful completion of account syncing process. |


#### AccountManagerObject

| Property          | Required | Type      | Description                                               |
| ----------------- | -------- | --------- | --------------------------------------------------------- |
| accounts          | ✔        | Account[] | Account objects.                                          |
| add_account()     | ✔        | function  | Adds a new account.                                       |
| remove_account()  | ✔        | function  | Removes an account.                                       |
| sync_accounts()   | ✔        | function  | Syncs all stored accounts with the Tangle.                |
| move()            | ✔        | function  | Inititates an internal transaction between accounts.      |
| backup()          | ✔        | function  | Creates a backup to a provided destination.               |
| import_accounts() | ✔        | function  | Imports backed up accounts.                               |
| get_account()     | ✔        | function  | Returns the account associated with the provided address. |
| reattach()        | ✔        | function  | Reattaches an unconfirmed transaction.                    |


#### Address 

Useful [reference](https://medium.com/@harshagoli/hd-wallets-explained-from-high-level-to-nuts-and-bolts-9a41545f5b0) for address management in Hierarchical Deterministic (HD) wallets.

| Property | Required | Type    | Description                                                                                                           |
| -------- | -------- | ------- | --------------------------------------------------------------------------------------------------------------------- |
| address  | ✔        | string  | Address (Bech32) string.                                                                                              |
| balance  | ✔        | number  | Address balance.                                                                                                      |
| index    | ✔        | number  | Address index.                                                                                                        |
| internal | ✔        | boolean | Determines if an address is a public or an internal (change) address. See the concept of chain node for more details. |
| checksum | ✔        | string  | Address checksum.                                                                                                     |


#### Node

| Property | Required | Type                                                          | Description                                                  |
| -------- | -------- | ------------------------------------------------------------- | ------------------------------------------------------------ |
| url      | ✔        | string                                                        | Node URL.                                                    |
| pow      | ✔        | boolean                                                       | Determines if the node accepts proof of work.                |
| username | ✘        | string                                                        | Node username. Only required if node requires authorisation. |
| password | ✘        | string                                                        | Node password. Only required if node requires authorisation. |
| network  | ✔        | `mainnet` \| `devnet` \| `comnet` | IOTA public network name.                                    |

#### Timestamp

| Property                    | Required | Type     | Description                                                                             |
| --------------------------- | -------- | -------- | --------------------------------------------------------------------------------------- |
| format(type: string):string | ✔        | function | Transaction timestamp in various formats.<br/> For example: MM-DD-YYYY, DD MM YYYY hh:mm:ss. |

#### Transfer 

Transfer object required for creating a transaction. It allows end-users to specify the transaction amount and recipient address.

:::info
Currently, it is not possible to send multiple payloads as part of the message. That is why the _tag_ property is omitted from this interface. You can find more details in this [GitHub pull request](https://github.com/iotaledger/protocol-rfcs/pull/18#discussion_r468432794).
:::

| Property       | Required | Type               | Description                    |
| -------------- | -------- | ------------------ | ------------------------------ |
| amount         | ✔        | number             | Transfer amount.               |
| address        | ✔        | string             | Transfer address.              |
| indexation_key | ✘        | Indexation Payload | (Optional) Indexation payload. |


#### Value

| Property                     | Required | Type     | Description                      |
| ----------------------------- | -------- | -------- | -------------------------------- |
| with_denomination():string    | ✔        | function | Transaction amount with unit.    |
| without_denomination():number | ✔        | function | Transaction amount without unit. |


#### Input

| Property     | Required | Type   | Description                                        |
| ------------ | -------- | ------ | -------------------------------------------------- |
| type         | ✔        | number | Input type. Defaults to `0`.                       |
| id           | ✔        | string | BLAKE2b-256 hash of the transaction.               |
| output_index | ✔        | number | Index of the output on the referenced transaction. |

#### OutputAddress

| Property | Required | Type   | Description                                                  |
| -------- | -------- | ------ | ------------------------------------------------------------ |
| type     | ✔        | number | Set to value `0` to denote an Ed25519 address.               |
| address  | ✔        | string | If type is set to `0`, it should contain an Ed25519 address. |


#### Output

| Property | Required | Type          | Description                   |
| -------- | -------- | ------------- | ----------------------------- |
| type     | ✔        | number        | Output type. Defaults to `0`. |
| address  | ✔        | OutputAddress | Output address.               |
| amount   | ✔        | number        | Amount of tokens to deposit.  |

#### UnsignedDataPayload

| Property | Required | Type   | Description                                   |
| -------- | -------- | ------ | --------------------------------------------- |
| type     | ✔        | number | Set to `2` to denote a unsigned data payload. |
| data     | ✔        | string | Data of unsigned payload.                     |


#### SignedDataPayload

| Property   | Required | Type   | Description                                      |
| ---------- | -------- | ------ | ------------------------------------------------ |
| type       | ✔        | number | Set to `3` to denote a signed data payload.      |
| data       | ✔        | string | Data of signed data payload.                     |
| public_key | ✔        | string | Ed25519 public key used to verify the signature. |
| signature  | ✔        | string | Signature of signing data.                       |


#### IndexationPayload

| *Property* | *Required* | *Type* | *Description*    |
| ---------- | ---------- | ------ | ---------------- |
| index      | ✔          | string | Indexation key.  |
| data       | ✔          | string | Indexation data. |

#### UnsignedTransaction

| Property       | Required | Type                                                                                                                                                             | Description                                                                                                                             |
| -------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| type           | ✔        | number                                                                                                                                                           | Transaction type. Defaults to 0.                                                                                                        |
| inputs_count   | ✔        | number                                                                                                                                                           | Amount of inputs proceeding.                                                                                                            |
| inputs         | ✔        | Input[]                                                                                                                                                          | Transaction inputs.                                                                                                                     |
| outputs_count  | ✔        | number                                                                                                                                                           | Amount of outputs proceeding.                                                                                                           |
| outputs        | ✔        | Output[]                                                                                                                                                         | Output address.                                                                                                                         |
| payload_length | ✔        | number                                                                                                                                                           | Length of optional payload.                                                                                                             |
| payload        | ✔        | [UnsignedDataPayload](#unsigneddatapayload) \| [SignedDataPayload](#signeddatapayload) \| [IndexationPayload](#indexationpayload) | Payload containing data. As multiple payloads are not yet supported, only [unsigned data payload](#unsigneddatapayload) should be used. |

#### Ed25519Signature

| *Property* | *Required* | *Type* | *Description*                                                            |
| ---------- | ---------- | ------ | ------------------------------------------------------------------------ |
| type       | ✔          | number | Set to value `1` to denote an Ed25519 signature.                         |
| public_key | ✔          | number | Public key of the Ed25519 keypair which is used to verify the signature. |
| signature  | ✔          | string | Signature signing the serialized unsigned transaction.                   |


#### SignatureUnblockBlock

| *Property* | *Required* | *Type*           | *Description*                                               |
| ---------- | ---------- | ---------------- | ----------------------------------------------------------- |
| type       | ✔          | number           | Set to value `0` to denote a signature unlock block.        |
| signature  | ✔          | Ed25519Signature | An unlock block containing signature(s) unlocking input(s). |


#### ReferenceUnblockBlock

| *Property* | *Required* | *Type* | *Description*                                        |
| ---------- | ---------- | ------ | ---------------------------------------------------- |
| type       | ✔          | number | Set to value `1` to denote a reference unlock block. |
| reference  | ✔          | number | Index of a previous unlock block.                    |

#### SignedTransactionPayload

| *Property*           | *Required* | *Type*                | *Description*                                                                                    |
| -------------------- | ---------- | --------------------- | ------------------------------------------------------------------------------------------------ |
| type                 | ✔          | number                | Payload type. Defaults to `0`.                                                                   |
| transaction          | ✔          | UnsignedTransaction   | Essence data making up a transaction by defining its inputs and outputs and an optional payload. |
| unblock_blocks_count | ✔          | number                | Number of inputs specifed.                                                                       |
| unblock_blocks       | ✔          | SignatureUnblockBlock | ReferenceUnblockBlock                                                                            | Holds the unlock blocks unlocking inputs within an Unsigned Transaction |

#### Message

| Property                                    | Required                                                               | Type                                                  | Description                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| version                                     | ✔                                                                      | number                                                | Message version. Defaults to `1`.                                                                                                                                                                                                                                                                                                |
| parents                                     | ✔                                                                      | string[]                                              | Message ids this message references.                                                                                                                                                                                                                                                                                             |
| payload_length                              | ✔                                                                      | number                                                | Length of the payload.                                                                                                                                                                                                                                                                                                           |
| payload                                     | ✔                                                                      | [SignedTransactionPayload](#signedtransactionpayload) |
| [UnsignedDataPayload](#unsigneddatapayload) |
| [SignedDataPayload](#signeddatapayload)     | Transaction amount (exposed as a custom type with additional methods). |
| timestamp                                   | ✔                                                                      | Timestamp                                             | Transaction timestamp (exposed as a custom type with additional methods).                                                                                                                                                                                                                                                        |
| nonce                                       | ✔                                                                      | string                                                | Transaction nonce.                                                                                                                                                                                                                                                                                                               |
| confirmed                                   | ✔                                                                      | boolean                                               | Determines if the transaction is confirmed.                                                                                                                                                                                                                                                                                      |
| broadcasted                                 | ✔                                                                      | boolean                                               | Determines if the transaction was broadcasted to the network. This will be true if the transaction was fetched from the network or if the transaction was successfully broadcasted from the client itself. This property may only be required for clients with persistent storage. |
| incoming                                    | ✔                                                                      | boolean                                               | Determines if the message is an incoming transaction or not.                                                                                                                                                                                                                                                                     |
| value                                       | ✔                                                                      | number                                                | Message transfer value.                                                                                                                                                                                                                                                                                                          |


#### StorageAdapter

| Property                               | Required | Type     | Description                                              |
| -------------------------------------- | -------- | -------- | -------------------------------------------------------- |
| get(key: string):Account               | ✔        | function | Gets the account object for provided account name or ID. |
| getAll(): Account[]                    | ✔        | function | Gets all account objects from storage.                   |
| set(key: string, payload: string):void | ✔        | function | Stores account in storage.                               |
| remove(key: string): void              | ✔        | function | Removes account from storage.                            |

## Storage 

:::warning
Using Stronghold for storage is currently under research/development.
:::

You should consider multiple storage options should for managing data that requires persistence:
- You can use a simple key-value [storage](https://capacitor.ionicframework.com/docs/apis/storage/) could be leveraged for wallet basic metadata, such as user settings or theming options. 
- For transactions and address data management you could use a relational database such as [SQLite](https://github.com/jepiqueau/capacitor-sqlite). 

What follows is an Entity Relationship Diagram (ERD)  that shows the logical representation of the data. An _account_ is the basic entity in this database design. It has a one-to-many relationship with _addresses_. This means an account could have multiple addresses , but an address can only belong to a single account. An account has a many-to-many relationship with _transactions_ .  Therefore, an account could have multiple transactions, but it is possible that a transaction belongs to multiple accounts. To accommodate this behaviour, an additional table that stores account IDs against transaction IDs (hashes) was added.  

A _storage adapter_ is required by the Rust layer to handle all the storage operations (read/write) from that layer. A generic storage adapter is defined in the [storage adapter section](#storage-adapter).  

![Storage - Entity Relationship Diagram](/img/specs/erdIOTA.svg)

## Storage Adapter

The package should have a default opinionated storage mechanism but should also provide the ability for users to override the storage by specifying an adapter. As a default option, a relational database such as [SQLite](https://www.sqlite.org/index.html) can be used.  

See [storage adapter](#storageadapter) for adapter interface.

## Account

### API

#### Initialisation 

Initializes account
There are several scenarios in which an account can be initialized:

- *Seed generated outside the Stronghold*: In this case, the _account_ should be initialized with a seed. It should communicate with the Stronghold using the `import_accounts` method and should expect an ID as a response. 
- *Seed generated inside the Stronghold*: In this case, the _account_ should be initialized without a seed. It should communicate with the Stronghold using its `create_account` method and should expect an “id” in response;
- *Importing accounts from Stronghold backup*: In this case, the _account_ should receive all the initialization properties from the `Stronghold`. Please note that during backup, these configuration settings should be passed to the `Stronghold`. See [import_accounts()](#import_accounts).

The following should be considered when initializing an account:

- An _account_ should never be initialized directly. The only way an _account_ can be initialized is through the [add_account()](#add_account) method.
- An _account_ should always be initialized after a successful response from the `Stronghold`. If the `Stronghold` fails to create an _account_ , the _account_ initialization should error out. If the `Stronghold` successfully creates an _account_ , the _account_ should be stored in the persistent storage. Upon a successful store operation, the user should be returned an _account_ object.
- If a _provider_ property is not passed, a random node should be selected from the _nodes_ property.
- If a _type_ property is not passed, _default_ should be used as an account type.
- _quorum_size_ and _quorum_threshold_ should be validated. For example, _quorum_size_ should not be greater than the number of nodes provided by the user.
- The _nodes_ property should validate and remove duplicate node URLs.
- All the properties of the returned account object should be read-only. It should not be possible to manipulate them directly.

##### Parameters

| *Name* | *Required* | *Type*        | *Description*                                          |
| ------ | ---------- | ------------- | ------------------------------------------------------ |
| config | ✔          | AccountConfig | Initialization method receives a configuration object. |

##### Returns

| *Name*  | *Type*  | *Description*     |
| ------- | ------- | ----------------- |
| account | Account | Account instance. |

##### Additional Information
| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### check_for_new_used_addresses() 

Sync addresses with the Tangle. The method ensures that the wallet's local state contains all used addresses and an unused address. 
 
The following should be considered when implementing this method:

- The updated address history should not be written down in the database/persistent storage. Instead, the method should only return the updated address history (with transaction hashes).  This ensures that there are no partial writes to the database.
- To sync addresses for an account from scratch, _gap_limit = 10_ should be sent as arguments.
*   To sync addresses from the latest address, _gap_limit = 1_ should be sent as arguments. 

##### Parameters

| *Name*    | *Required* | *Type* | *Description*                                                                                         |
| --------- | ---------- | ------ | ----------------------------------------------------------------------------------------------------- |
| gap_limit | ✔          | number | Number of address indexes that are generated.                                                         |


##### Returns

| *Name*    | *Type*    | *Description*                                |
| --------- | --------- | -------------------------------------------- |
| addresses | Address[] | Address history up to latest unused address. |
| ids       | string[]  | Message IDs associated with the addresses.   |

##### Additional Information


      
| *Name*                          | *Description*                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Access modifiers                | Private                                                                                                                                                                                                                                                                                                                                                                                                            |
| Errors                          | List of error messages [TBD]                                                                                                                                                                                                                                                                                                                                                                                       |
| Required client library methods | [get_address_balances()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#get_address_balances)\| [find_messages()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages) \| [find_outputs()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_outputs) |

#### sync_addresses_and_messages() 

Sync messages with the Tangle. The method should ensure that the wallet's local state has all messages associated with the address history. 

The following should be considered when implementing this method:

- The updated message history should not be written down in the database/persistent storage. Instead, the method should only return the updated message history (with message IDs).
- This method should check if there are any local messages (with “broadcasted: false”) matching the messages fetched from the network. If there are such messages, their “broadcasted” property should be set to true.
- For newly-confirmed messages, the method should ensure that it updates the “confirmed” property of all its reattachments.


##### Parameters

| *Name* | *Required* | *Type*   | *Description*                                                                                                                         |
| ------ | ---------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| ids    | ✔          | string[] | Message IDs. New message IDs should be calculated by running a difference of local message IDs with latest message IDs on the Tangle. |


##### Returns

| *Name*   | *Type*                | *Description*   |
| -------- | --------------------- | --------------- |
| messages | [Message](#message)[] | Message history |

##### Additional Information

| *Name*                          | *Description*                                                                                                          |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Access modifiers                | Private                                                                                                                |
| Errors                          | List of error messages [TBD]                                                                                           |
| Required client library methods | [find_messages()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages) |

##### Required client library methods
- 

#### select_inputs() 

Select inputs for funds transfer.

:::info
This method should only be used internally by [send()](#send). The input selection method should also ensure that the recipient address doesn't match the remainder address. 
:::

See [Input Selection Process](#input-selection) for implementation details.

##### Parameters

| *Name*    | *Required* | *Type* | *Description*               |
| --------- | ---------- | ------ | --------------------------- |
| threshold | ✔          | number | Amount user wants to spend. |
| address   | ✔          | string | Recipient address.          |


##### Returns

| *Name*    | *Type*    | *Description*                                                              |
| --------- | --------- | -------------------------------------------------------------------------- |
| inputs    | Address[] | Selected Inputs                                                            |
| remainder | Address   | Remainder address object. Empty or null if there's no need for a remainder |


##### Additional Information

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Private                      |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### send() 

Sends a message to the Tangle.  

:::info
This method should only be used after a successful response from [sync()](#sync). 
:::

Currently, it is not possible to send multiple payloads. 

If you want to send a value transaction, please follow this process:
1. Ensure _amount_ is not set to zero.
2. Ensure _amount_ does not exceed the total balance.
3. Ensure recipient address has correct checksum.
4. Validate _data_ property semantics and size.
5. Select inputs by using [select_inputs()](#select_inputs).
6. Pass the serialized [unsigned transaction](#unsignedtransaction) to the `Stronghold` for signing with its “signTransaction” method.
7. Perform proof-of-work. The _pow_ property in the account object should determine if the proof of work should be offloaded.
8. Once proof-of-work is successfully performed, the message should be validated and stored in the persistent storage.
9. After persisting the transaction, the transaction should be broadcast to the network.
10.  In the event of a broadcast error, there should be three attempts for automatic rebroadcasting. If all attempts fail, the send process should terminate, and it should be left to the user to retry the failed message. For failed messages, the “broadcasted” property in the transaction objects should be set to false. 


##### Parameters

| *Name*   | *Required* | *Type*   | *Description*    |
| -------- | ---------- | -------- | ---------------- |
| transfer | ✔          | Transfer | Transfer object. |


##### Returns

| *Name*  | *Type*              | *Description*       |
| ------- | ------------------- | ------------------- |
| message | [Message](#message) | Newly made message. |


##### Additional Information 

| *Name*                          | *Description*                                                                                                                                                                                                              |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Access modifiers                | Private                                                                                                                                                                                                                    |
| Errors                          | List of error messages [TBD]                                                                                                                                                                                               |
| Required client library methods | [find_messages()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages) \| [send()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#send) |


#### retry() 

Rebroadcasts failed message.

:::info
This method should only be used after a successful response from [sync()](#sync). 
:::

If you want to retry broadcasting a failed message, you can use the following process:

1. Get the message by using [get_message()](#get_message).
2. Rebroadcast the message.
3. Update the account in persistent storage.

##### Parameters

| *Name* | *Required* | *Type* | *Description* |
| ------ | ---------- | ------ | ------------- |
| id     | ✔          | string | Message ID    |

##### Returns

| *Name*  | *Type*  | *Description*       |
| ------- | ------- | ------------------- |
| message | Message | Newly made message. |


##### Additional Information

| *Name*                          | *Description*                                                                                                        |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Access modifiers                | Private                                                                                                              |
| Errors                          | List of error messages [TBD]                                                                                         |
| Required client library methods | [post_message()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#post_message) |


#### sync()

Syncs an account with the Tangle. The account syncing process should ensure that the latest metadata (balance, messages) associated with an account is retrieved from the Tangle and stored locally.  
Please note that it is a proposed design decision to enforce account syncing before every send. An alternative way would be to have the _send_ method always exposed and internally ensuring that the account is synced before every message. 

If you want to sync an account, you can use the following process:

1. Sync addresses using [check_for_new_used_addresses()](#check_for_new_used_addresses).
2. Sync messages using [sync_addresses_and_messages()](#sync_messages).
3. Store updated addresses and messages information in persistent storage (if not explicitly set otherwise by the user). 

##### Parameters

| *Name*           | *Required* | *Type*  | *Description*                                                                                                                                                                                                                                                                                               |
| ---------------- | ---------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| index            | ✘          | number  | Address index. By default the number of addresses stored for this account should be used as an index.                                                                                                                                                                                                       |
| gap_limit        | ✘          | number  | Number of address indexes that are generated.                                                                                                                                                                                                                                                               |
| skip_persistence | ✘          | boolean | Skips write to the database. This will be useful if a user wants to scan the Tangle for further addresses to find balance. You can find more details in the [snapshot transition feature](https://legacy.docs.iota.org/docs/wallets/0.1/trinity/how-to-guides/perform-a-snapshot-transition) provided by Trinity. |


##### Returns

| *Name*  | *Type*                                | *Description*          |
| ------- | ------------------------------------- | ---------------------- |
| account | [SyncedAccount](#syncedaccountobject) | Synced account object. |


##### Additional Information

| *Name*                          | *Description*                                                                                                                                                                                                                                              |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Access modifiers                | Public                                                                                                                                                                                                                                                     |
| Errors                          | List of error messages [TBD]                                                                                                                                                                                                                               |
| Required client library methods | [find_messages()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages) \| [get_address_balances()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#get_address_balances) |

####  reattach() 

Reattaches unconfirmed message to the Tangle. 
The following should be considered when implementing this method:

- Only an unconfirmed message can be reattached. The method should validate the confirmation state of the provided transaction. If a confirmed message ID is provided, the method should throw an error.
- The method should also validate if a reattachment is necessary, by checking if the message falls below max depth. The criteria for whether the message has fallen below max depth is determined through its timestamp. If 11 minutes have passed since the timestamp of the most recent reattachment, the message can be reattached. See [this implementation](https://github.com/iotaledger/trinity-wallet/blob/3fab4f671c97e805a2b0ade99b4abb8b508c2842/src/shared/libs/iota/transfers.js#L141) for reference.
- Once reattached, the message should be stored in the persistent storage.
- If the message was reattached via polling, a [reattachment](#monitor-for-reattachments) event should be emitted to notify all subscribers. 


##### Parameters

| *Name* | *Required* | *Type* | *Description* |
| ------ | ---------- | ------ | ------------- |
| id     | ✔          | string | Message ID.   |


##### Returns

| *Name*  | *Type*  | *Description*             |
| ------- | ------- | ------------------------- |
| message | Message | Newly reattached message. |

##### Additional Information

| *Name*                          | *Description*                                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Access modifiers                | Public                                                                                                       |
| Errors                          | List of error messages [TBD]                                                                                 |
| Required client library methods | [reattach()](https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#reattach) |


#### total_balance()

Gets total account balance.

The total balance should be read directly from local storage. To read the latest account balance from the network, [sync()](#sync) should be used first. 

##### Returns

| *Type* | *Description*          |
| ------ | ---------------------- |
| Value  | Account total balance. |


##### Additional Information

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### available_balance()

Gets available account balance. The available account balance is the amount users are allowed to spend. It should subtract the pending balance from the total balance. 

For example, if a user with _50i_ total account balance has made a transaction spending _30i_, the available balance should be _20i_ (i.e. 50i - 30i).

The available balance should be read directly from local storage. If you want to read the latest account balance from the network, you should use [sync()](#sync) first.

##### Returns

| *Type* | *Description*                   |
| ------ | ------------------------------- |
| Value  | The accounts available balance. |


##### Additional Information

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### set_alias() 

Updates an account's alias/name.

##### Parameters

| *Name* | *Required* | *Type* | *Description*     |
| ------ | ---------- | ------ | ----------------- |
| alias  | ✔          | string | New account name. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### list_messages() 

Gets messages. Messages should be read directly from local storage. To ensure the local database is updated with the latest messages, you should use [sync()](#sync) first.

##### Parameters

| *Name* | *Required* | *Type* | *Description*                                                                                                                |
| ------ | ---------- | ------ | ---------------------------------------------------------------------------------------------------------------------------- |
| count  | ✔          | number | Number of (most recent) messages.                                                                                            |
| from   | ✔          | number | Subset of messages. For example: count = 10, from = 5, it should return ten messages skipping the most recent five messages. |

##### Returns

| *Name*   | *Type*    | *Description* |
| -------- | --------- | ------------- |
| messages | Message[] | All messages. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### list_received_messages()

Gets all received messages.

Messages should be read directly from local storage. To ensure the local database is updated with the latest messages, you should use [sync()](#sync) first. 

##### Parameters

| *Name* | *Required* | *Type* | *Description*                            |
| ------ | ---------- | ------ | ---------------------------------------- |
| count  | ✔          | number | Number of most recent received messages. |
| from   | ✔          | number | Subset of received messages.             |

##### Returns

| *Name*   | *Type*    | *Description*          |
| -------- | --------- | ---------------------- |
| messages | Message[] | All received messages. |


##### Additional Information 
| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### list_sent_messages()

Gets all sent messages.

Messages should be read directly from local storage. To ensure the local database is updated with the latest messages, you should use [sync()](#sync) first. 

##### Parameters

| *Name* | *Required* | *Type* | *Description*                          |
| ------ | ---------- | ------ | -------------------------------------- |
| count  | ✔          | number | Number of (most recent) sent messages. |
| from   | ✔          | number | Subset of sent messages.               |


##### Returns 

| *Name*   | *Type*    | *Description*      |
| -------- | --------- | ------------------ |
| messages | Message[] | All sent messages. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### list_failed_messages()

Gets all failed (broadcasted = false) messages. Messages should be read directly from local storage.

##### Parameters

| *Name* | *Required* | *Type* | *Description*                            |
| ------ | ---------- | ------ | ---------------------------------------- |
| count  | ✔          | number | Number of (most recent) failed messages. |
| from   | ✔          | number | Subset of failed messages.               |


##### Returns

| *Name*   | *Type*    | *Description*        |
| -------- | --------- | -------------------- |
| messages | Message[] | All failed messages. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### list_unconfirmed_messages()

Gets all unconfirmed (confirmed = false) messages. Messages should be read directly from local storage.  

##### Returns

| *Name* | *Required* | *Type* | *Description*                                 |
| ------ | ---------- | ------ | --------------------------------------------- |
| count  | ✔          | number | Number of (most recent) unconfirmed messages. |
| from   | ✔          | number | Subset of unconfirmed messages.               |


##### Returns

| *Name*   | *Type*    | *Description*             |
| -------- | --------- | ------------------------- |
| messages | Message[] | All unconfirmed messages. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### get_message()

Gets message for provided ID.

Messages should be read directly from local storage.  To ensure the local database is updated with the latest messages, you should use [sync()](#sync) first.

##### Parameters

| *Name* | *Required* | *Type* | *Description* |
| ------ | ---------- | ------ | ------------- |
| id     | ✔          | string | Message ID.   |


##### Returns 

| *Name*  | *Type*  | *Description*   |
| ------- | ------- | --------------- |
| message | Message | Message object. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### list_addresses()

Gets all addresses.

##### Returns

| *Name*    | *Type*                | *Description*  |
| --------- | --------------------- | -------------- |
| addresses | [Address](#address)[] | All addresses. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### list_unspent_addresses()

Gets all unspent input addresses

##### Returns

| Name      | *Type*                  | *Description*                |
| --------- | ----------------------- | ---------------------------- |
| addresses | [Address](#address)[] | All unspent input addresses. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### generate_address()

Gets the latest unused address.

##### Returns

| *Name*  | *Type*  | *Description*         |
| ------- | ------- | --------------------- |
| address | Address | A new address object. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

## Account Manager

An account manager class should be publicly available for users. With the account manager, the user can create, update, delete or manage multiple accounts. The implementation details of a specific account should be abstracted using this account manager wrapper. 

### API

#### Initialisation 

Initializes the account manager. Account manager initialization should validate the adapter object semantics and return an _AccountManager_ instance.

##### Parameters

| *Name*  | *Required* | *Type*                     | *Description*                                               |
| ------- | ---------- | -------------------------- | ----------------------------------------------------------- |
| adapter | ✘          | [Adapter](#storageadapter) | Initialisation method receives an optional storage adapter. |


##### Returns 

| *Name*  | *Type*                                   | *Description*             |
| ------- | ---------------------------------------- | ------------------------- |
| manager | [AccountManager](#accountmanagerobject) | Account manager instance. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |

#### add_account()

Adds new account

See account [initialisation](#initialisation) for detailed implementation guidelines.

##### Parameters

| *Name* | *Required* | *Type*                                 | *Description*                 |
| ------ | ---------- | -------------------------------------- | ----------------------------- |
| config | ✔          | [AccountConfig](#accountconfiguration) | Account configuration object. |

##### Returns

| *Name*   | *Type*              | *Description*          |
| -------- | ------------------- | ---------------------- |
| accounts | [Account](#account) | Newly created account. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### remove_account()

Removes an account.

The following should be considered when removing an account:
- An account should first be removed from the  `Stronghold`  using its _removeAccount_ method.
- Once the account references have been removed from the `Stronghold`, the account should be deleted from the persistent storage.


##### Parameters

| *Name*     | *Required* | *Type*                                                                                                                                                       | *Description*                                            |
| ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| identifier | ✔          | &#x7B; address: &lt;string> } \| &#x7B; alias: &lt;string> }  \| &#x7B; ID: &lt;number> } \| &#x7B; index: &lt;number } | Identifier. Could be one of address, alias, ID or index. |

##### Additional Information

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### sync_accounts() 

Syncs all stored accounts with the Tangle. Syncing should get the latest balance for all accounts and should find any new messages associated with the stored account.

See [Accounts Syncing Process](#account-syncing-process) for further details.

##### Returns

| *Name*  | *Type*                                  | *Description*    |
| ------- | --------------------------------------- | ---------------- |
| account | [SyncedAccount](#syncedaccountobject)[] | Synced accounts. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | [sync()](#sync)              |


#### move()

Moves funds from one account to another. This method should use the [send()](#send) method from the sender account and initiate a message to the receiver account.

##### Parameters

| *Name* | *Required* | *Type*                                                                                                                                                       | *Description*                                            |
| ------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| from   | ✔          | &#x7B; address: &lt;string> } \|   &#x7B; alias: &lt;string> } \| &#x7B; ID: &lt;number>\| &#x7B; index: &lt;number> }    | Identifier. Could be one of address, alias, ID or index. |
| to     | ✔          | &#x7B; address: &lt;string> } \|   &#x7B; alias: &lt;string> } \|  &#x7B; ID: &lt;number> } \| &#x7B; index: &lt;number> } | Identifier. Could be one of address, alias, ID or index. |
| amount | ✔          | number                                                                                                                                                       | Transaction amount.                                      |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### backup()

Safely creates a backup of the accounts to a destination. The file could simply be JSON containing the address & transaction histories for accounts.

This method should provide the `Stronghold` instance with the metadata of all accounts. 

##### Parameters

| *Name*      | *Required* | *Type* | *Description*                           |
| ----------- | ---------- | ------ | --------------------------------------- |
| destination | ✔          | string | Path where the backup should be stored. |

##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### import_accounts

Import (backed up) accounts.

**The implementation details are not finalized.**

##### Parameters

| *Name*   | *Required* | *Type*                | *Description*   |
| -------- | ---------- | --------------------- | --------------- |
| accounts | ✔          | [Account](#account)[] | Account object. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### get_account() 

Returns the account associated with the provided identifier.


##### Parameters

| *Name*     | *Required* | *Type*                                                                                                                                                     | *Description*                                            |
| ---------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| identifier | ✔          | &#x7B; address: &lt;string> } \|  &#x7B; alias: &lt;string>  } \| &#x7B; ID: &lt;number> } \| &#x7B; index: &lt;number> } | Identifier. Could be one of address, alias, ID or index. |


##### Returns

| *Name*  | *Type*              | *Description*                       |
| ------- | ------------------- | ----------------------------------- |
| account | [Account](#account) | Account associated with identifier. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


#### reattach()

Reattaches an unconfirmed message.

See [reattach()](#reattach) method for implementation details. This method is a wrapper method provided for convenience. A user could directly access the [reattach()](#reattach) method on an account object. 


##### Parameters

| *Name*     | *Required* | *Type*                                                                                                                                                       | *Description*                                            |
| ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| identifier | ✔          | &#x7B; address: &lt;string> } \|  &#x7B; alias: &lt;string> } \| &#x7B; ID: &lt;number> } \| &#x7B; index: &lt;number }  | Identifier. Could be one of address, alias, ID or index. |
| id         | ✔          | string                                                                                                                                                       | Message ID.                                              |



##### Returns 

| *Name*  | *Type*              | *Description*             |
| ------- | ------------------- | ------------------------- |
| message | [Message](#message) | Newly reattached message. |


##### Additional Information 

| *Name*                          | *Description*                |
| ------------------------------- | ---------------------------- |
| Access modifiers                | Public                       |
| Errors                          | List of error messages [TBD] |
| Required client library methods | None                         |


## Events 

Events can have two categories:

1. Reactive messages emitted from the node software whenever the state on the node changes. For example, emitting new messages received by the node. Clients (Wallet) can subscribe to these events to get notified if any relevant change occurs on the node. For further details, please visit the [Firefly GitHub repository](https://github.com/iotaledger/wallet-spec/tree/events).
   
2. Messages emitted from the wallet library whenever there are any important state changes. Please note that in cases where a user triggered action leads to a state change, the messages will not be emitted. For example, if a user explicitly triggers a [sync()](#sync) action leading to a state change, an explicit event is not necessary.

### Category 1 events

On every update sent from the node software via an event, the wallet library should update internal (persistent) storage and should also emit events via [category 2 events](#category-2-events). 

#### Monitor address for balance changes

| *Event*                 | *Returned Data*                                                                |
| ----------------------- | ------------------------------------------------------------------------------ |
| &lt; Address : Balance> | Index 1: Address \| Index 2: New balance on the address |
    
#### Monitor address for new messages 

| *Event*                | *Returned Data*                                                           |
| ---------------------- | ------------------------------------------------------------------------- |
| &lt;Address : Message> | Index 1: Address \| Index 2: Id of the new message |

#### Monitor message for confirmation state 

| *Event*        | *Returned Data*                                                           |
| -------------- | ------------------------------------------------------------------------- |
| &lt;MessageId> | Index 1: Message Id | Index 2: Confirmation state |
    
### Category 2 events

They could be triggered via events from [category 1](#category-1-events) or through [polling](#polling). 

#### Monitor for balance changes

| *Event*  | *Returned Data*                   |
| -------- | --------------------------------- |
| balances | [{ accountId, address, balance }] |


#### Monitor for new messages 

| *Event*  | *Returned Data*           |
| -------- | ------------------------- |
| messages | [{ accountId, messages }] |


#### Monitor for confirmation state 

| *Event*       | *Returned Data*            |
| ------------- | -------------------------- |
| confirmations | [{ accountId, messages  }] |

#### Monitor for reattachments 

| *Event*       | *Returned Data*            |
| ------------- | -------------------------- |
| reattachments | [{ accountId, messages  }] |


#### Monitor for broadcasts 

| *Event*    | *Returned Data*            |
| ---------- | -------------------------- |
| broadcasts | [{ accountId, messages  }] |

#### Monitor for errors 

| *Event* | *Returned Data*  |
| ------- | ---------------- |
| error   | { type, error  } |

## Privacy

To maintain the financial privacy of wallet users, you should enforce strategies in the application/wallet that will guarantee a certain level of anonymity:

- The wallet should only use a single address per message.  If an address has already been used in a message, it should not be used as a deposit address.  Instead, a new address should be generated.- The input selection strategy should expose as little information as possible. Please see the [input selection process](#input-selection) for further details.

Some other privacy enhancing techniques can be found [in this document](https://docs.google.com/document/d/1frk4r1Eq4hnGGOiKWkDiGTK5QQxKbfrvl7Iol7OZ-dc/edit#). 

## Input Selection

The goal of input selection is to avoid remainder addresses. The remainder output leaves a clue to the user's future spends. There should be a standardized input selection strategy used by the wallet. 

The steps for input selection are as follows:

1. Try to select an input with an exact match. For example, if a user intends to spend _X_ iotas, the wallet should try to find an address that has _X_ iotas as available balance.
2. If the previous step fails, try to select a combination of inputs that satisfy the amount leaving no change. For example, consider a scenario where the wallet with account name _Foo_ has three addresses _A_, _B_ and _C_ with _10_, _20_ and _50_ IOTA respectively. If a user intends to spend _X = 30_ IOTA, the application should search for an exact match (step no. 1). In this case, no address balance matches _X_. Therefore, the wallet should search for a subset of addresses with an accumulated balance of _X_. In this scenario, _A_ and _B_.
3. If both the previous steps fail, the wallet should select a combination of inputs that produce the minimum remainder. 

A reference implementation of different input selection algorithms for Bitcoin can be found [in this project](https://github.com/bitcoinjs/coinselect).

The implementation of step no. 2 is also quite similar to the [subset sum problem](https://en.wikipedia.org/wiki/Subset_sum_problem). Given a _total_ and a set of non-negative numbers (_inputs_), we need to determine if there is a subset which adds up to the _total_.

## Account Syncing Process

The account syncing process should detect all used accounts on a seed with their corresponding address and message history. Once, all accounts and histories are detected, the wallet should accumulate the total balance. The syncing process should work as follows: 

1. Start with the account at index 0, generate [gap limit](https://blog.blockonomics.co/bitcoin-what-is-this-gap-limit-4f098e52d7e1) number of addresses. This defaults to 20.
2. Check for messages and balances on the generated addresses.
3. If there are no messages and balances of 0 on all addresses, the process for generating addresses and finding messages and balances should be stopped.
4. If any address has balance or associated messages, generate gap limit number of addresses from the index of the last address with messages or balance.
5. Steps (1-4) should also be performed for the account at index 1. The general idea is that _n + 1_ accounts should be checked if account _n_ has any messages or balance.

Treat accounts like addresses. Only allow 1 latest unused account.

*Scenario 1*: The wallet message and address history stored in Stronghold backup

1. Start syncing from the latest address index stored in the Stronghold backup
2.  Run the “Full sync” function to resync from index 0 across all accounts
3.  Run the “Find more history” function to sync a further 50 addresses

*Scenario 2*: User has no backup file

1. Start syncing from account 0 address 0

## Polling

 A background process that automatically performs several tasks periodically should be part of the wallet library. The goal of the background process is to perform the following tasks:  

- *Sync accounts*: The background process should sync all accounts with the network. This should be done using the [`sync_accounts()`](#sync_accounts) method. 
  - If new messages are detected, a [messages](#monitor-for-new-messages) event should be used to notify all subscribers. 
  - If new balances are detected, a [balances](#monitor-for-balance-changes) event should be used to notify all subscribers. 
  - If new confirmations are detected, a [confirmations](#monitor-for-confirmation-state) event should be used to notify all subscribers.
:::info
If there are multiple failed messages, priority should be given to the old ones. 
:::
- *Reattach*: The background process should check if there are any unconfirmed messages that require reattachment. The detailed implementation flow for reattachment can be found in the [reattach section](#reattach). 

The following should be considered for implementation:

*   Invoking a task explicitly that is already being performed through polling should lead to an error. For example, if the polling process is already syncing accounts, and a user explicitly calls [sync()](#sync), it should throw an error.
*   Errors during the polling process should be communicated to subscribers via error events.

The background process should have a recurring checker that sequentially performs all the above tasks. The implementation should ensure that future tasks can easily be added to the background process. For reference, see [Trinity's implementation](https://github.com/iotaledger/trinity-wallet/blob/develop/src/mobile/src/ui/components/Poll.js) of the poll component. 