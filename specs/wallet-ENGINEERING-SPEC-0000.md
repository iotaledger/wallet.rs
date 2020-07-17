# Wallet Library Spec

## Table of Contents
- [Wallet Library Spec](#wallet-library-spec)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Considerations](#considerations)
  - [Interfaces](#interfaces)
      - [Account Configuration Interface](#account-configuration-interface)
      - [Account Object Interface](#account-object-interface)
      - [Address](#address)
      - [Node](#node)
      - [Tag](#tag)
      - [Timestamp](#timestamp)
      - [Transfer](#transfer)
      - [Value](#value)
      - [Signature Message Interface](#signature-message-interface)
      - [Transaction](#transaction)
      - [Storage Adapter Interface](#storage-adapter-interface)
  - [Storage](#storage)
  - [Storage Adapter](#storage-adapter)
  - [Account](#account)
    - [API](#api)
      - [Initialisation](#initialisation)
      - [sync_addresses()](#sync_addresses)
      - [sync_transactions()](#sync_transactions)
      - [select_inputs()](#select_inputs)
      - [send()](#send)
      - [retry()](#retry)
      - [sync()](#sync)
      - [reattach()](#reattach)
      - [sendMessage()](#sendmessage)
      - [totalBalance()](#totalbalance)
      - [availableBalance()](#availablebalance)
      - [setAlias()](#setalias)
      - [listTransactions()](#listtransactions)
      - [listReceivedTransactions()](#listreceivedtransactions)
      - [listSentTransactions()](#listsenttransactions)
      - [listFailedTransactions()](#listfailedtransactions)
      - [listUnconfirmedTransactions()](#listunconfirmedtransactions)
      - [getTransaction()](#gettransaction)
      - [listAddresses()](#listaddresses)
      - [listUnspent()](#listunspent)
      - [generateAddress()](#generateaddress)
  - [Accounts Manager](#accounts-manager)
    - [API](#api-1)
      - [Initialisation - _Initialises accounts manager_](#initialisation---initialises-accounts-manager)
      - [addAccount()](#addaccount)
      - [removeAccount()](#removeaccount)
      - [syncAccounts()](#syncaccounts)
      - [move()](#move)
      - [backup()](#backup)
      - [importAccounts](#importaccounts)
      - [getAccount()](#getaccount)
      - [reattach()](#reattach-1)
  - [Events](#events)
      - [Monitor address for balance changes](#monitor-address-for-balance-changes)
      - [Monitor address for new transactions](#monitor-address-for-new-transactions)
      - [Monitor transaction for confirmation state](#monitor-transaction-for-confirmation-state)
      - [Monitor for balance changes](#monitor-for-balance-changes)
      - [Monitor for new transactions](#monitor-for-new-transactions)
      - [Monitor for confirmation state](#monitor-for-confirmation-state)
      - [Monitor for reattachments](#monitor-for-reattachments)
      - [Monitor for broadcasts](#monitor-for-broadcasts)
      - [Monitor for errors](#monitor-for-errors)
  - [Privacy](#privacy)
  - [Input Selection](#input-selection)
  - [Accounts Syncing Process](#accounts-syncing-process)
  - [Polling](#polling)

## Introduction

The wallet library is a stateful package with a standardised interface for developers to build applications involving IOTA value transactions. The package will be compatible with different platforms such as web, desktop and mobile. 

The package introduces the concept of an “account”. An account is a reference or a label to a [seed](https://docs.iota.org/docs/getting-started/0.1/clients/seeds). An account has certain properties such as [addresses](https://docs.iota.org/docs/getting-started/0.1/clients/addresses#) and [transactions](https://docs.iota.org/docs/getting-started/0.1/transactions/transactions). An account has various possible behaviours, including moving funds, looking for new transactions, and making copies of transaction histories. An account should also be able to provide a degree of financial/transaction privacy and this should not incur any overhead. 

A similar [package](https://docs.iota.org/docs/client-libraries/0.1/account-module/introduction/overview) was previously developed, introducing the concept of Conditional Deposit Addresses (CDAs), but this becomes obsolete with the introduction of Ed25 signatures. The previous account package was limited to a single account. As an improvement, the (new) package will be able to manage multiple accounts. 

To summarize, the main motivation behind this package is to offer a simplified (stateful) approach to handle IOTA payments.

## Considerations

*   The structure of some interfaces are not final and may be changed. For example, some of the properties in the [Transaction](#transaction) structure will be different with the introduction of Chrysalis.
*   Methods of some interfaces e.g., [Tag](#tag), [Timestamp](#timestamp), [Value](#value) and [SignatureMessageFragment](#signaturemessagefragment) may be offered as separate helper methods instead of embedding them in the [Transaction](#transaction) interface.
*   Seeds should be stored and managed separately in a secure enclave and should never leave the secure environment. Secure enclaves include software enclaves such as IOTA’s Rust-based Stronghold library or hardware enclaves such as a Ledger Nano.
*   The secure enclave should have the ability to generate addresses and sign transactions upon receipt of a message, and return the output in a message. If the secure enclave is provided with a pre-generated seed, the sender process should immediately remove the seed traces from memory. 

## Interfaces

#### Account Configuration Interface

Configuration or initialisation object. It should support parameters accepted by high level [client](https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY) libraries.

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>mnemonic</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>BIP-39 mnemonic. When importing an account from stronghold backup, the mnemonic will not be required.</td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10008;</td>
    <td>string
    </td>
    <td>SHA-256 hash of the first address on the seed (m/44'/0'/0'/0/0). Required for referencing a seed in stronghold.
      The id should be provided by stronghold. 
    </td>
  </tr>
  <tr>
    <td>index</td>
    <td>&#10004;</td>
    <td>number
    </td>
    <td>Account index in BIP-44 derivation path.</td>
  </tr>
  <tr>
    <td>alias</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Account name. If not provided, a `Account + ${index}` should be used. 
      When importing an account from stronghold backup, the alias will be required from stronghold.
    </td>
  </tr>
  <tr>
    <td>pow</td>
    <td>&#10008;</td>
    <td>‘local’ | ‘remote’
    </td>
    <td>Proof of work settings. Defaults to ‘local’. 
      ‘local’: Should be performed on device
      ‘remote’: Should be performed on the node
    </td>
  </tr>
  <tr>
    <td>nodes</td>
    <td>&#10004;</td>
    <td>
      <a href="#node">node</a>[]
    </td>
    <td>A list of nodes to connect to.
    </td>
  </tr>
  <tr>
    <td>quorumSize</td>
    <td>&#10008;</td>
    <td>number</td>
    <td>If multiple nodes are provided, quorum size determines the number of nodes to query to check for quorum.</td>
  </tr>
  <tr>
    <td>quorumThreshold</td>
    <td>&#10008;</td>
    <td>number
    </td>
    <td>Minimum number of nodes from the quorum pool that need to agree for considering the result as true.
    </td>
  </tr>
  <tr>
    <td>network?</td>
    <td>&#10008;</td>
    <td>‘mainnet’ | ‘devnet’ | ‘comnet’</td>
    <td>IOTA public network
    </td>
  </tr>
  <tr>
    <td>type?</td>
    <td>&#10008;</td>
    <td>‘default’ | ‘ledger’</td>
    <td>Account type. Would be required for differentiating ledger vs non-ledger accounts.</td>
  </tr>
  <tr>
    <td>provider?</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Node URL
    </td>
  </tr>
  <tr>
    <td>createdAt?</td>
    <td>&#10008;</td>
    <td>Date</td>
    <td>Time of account creation
    </td>
  </tr>
  <tr>
    <td>transactions?</td>
    <td>&#10008;</td>
    <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
    </td>
    <td>Transactions associated with seed. Accounts can be initialised with locally stored transactions.
    </td>
  </tr>
  <tr>
    <td>addresses?</td>
    <td>&#10008;</td>
    <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>[]
    </td>
    <td>Address history  associated with seed. Accounts can be initialised with locally stored address history.
    </td>
  </tr>
</table>

#### Account Object Interface

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>First address on the seed. Required for referencing a seed in secure enclave/storage.</td>
  </tr>
  <tr>
    <td>alias</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Account name.</td>
  </tr>
  <tr>
    <td>createdAt</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Account creation time.</td>
  </tr>
  <tr>
    <td>lastSyncedAt</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Aime when the account was last synced with the tangle</td>
  </tr>
</table>

#### Address 

_Useful [reference](https://medium.com/@harshagoli/hd-wallets-explained-from-high-level-to-nuts-and-bolts-9a41545f5b0) for address management in Hierarchical Deterministic (HD) wallets_

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong>
    </td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Address trytes.</td>
  </tr>
  <tr>
    <td>balance</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Address balance.</td>
  </tr>
  <tr>
    <td>index</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Address index.</td>
  </tr>
  <tr>
    <td>checksum</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Address checksum.</td>
  </tr>
</table>

#### Node

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>url</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Node URL.</td>
  </tr>
  <tr>
    <td>pow</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>Determines if the node accepts proof of work.</td>
  </tr>
  <tr>
    <td>username?</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Node username. Only required if node requires authorisation.</td>
  </tr>
  <tr>
    <td>password?</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Node password. Only required if node requires authorisation.</td>
  </tr>
  <tr>
    <td>network</td>
    <td>&#10004;</td>
    <td>‘mainnet’ | ‘devnet’ | ‘comnet’ </td>
    <td>IOTA public network name.</td>
  </tr>
</table>

#### Tag

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>asTrytes():string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction tag as trytes.</td>
  </tr>
  <tr>
    <td>asAscii():string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction tag as ascii.</td>
  </tr>
</table>

#### Timestamp

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>format(type: string):string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction timestamp in various formats. For example: MM-DD-YYYY, DD MM YYYY hh:mm:ss
    </td>
  </tr>
</table>

#### Transfer 

Transfer object required for creating a transaction. It allows end-users to specify the transaction amount and recipient address along with a message or a tag.

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>amount</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Transfer amount.</td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transfer address.</td>
  </tr>
  <tr>
    <td>tag?</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transfer tag.</td>
  </tr>
  <tr>
    <td>message?</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transfer message.</td>
  </tr>
</table>

#### Value

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>withDenomination():string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction amount with unit.</td>
  </tr>
  <tr>
    <td>withoutDenomination():number</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction amount without unit.</td>
  </tr>
</table>

#### Signature Message Interface

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>getSignature():string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction signature.</td>
  </tr>
  <tr>
    <td>getMessage():number</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Message extracted from signature.</td>
  </tr>
</table>

#### Transaction

Note: some of the transaction properties will be different.

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>hash</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction hash.</td>
  </tr>
  <tr>
    <td>signatureMessageFragment</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Signature of the private key.</td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction address.</td>
  </tr>
  <tr>
    <td>value</td>
    <td>&#10004;</td>
    <td><a href="#value">Value</a></td>
    <td>Transaction amount (exposed as a custom type with additional methods).</td>
  </tr>
  <tr>
    <td>tag</td>
    <td>&#10004;</td>
    <td><a href="#tag">Tag</a></td>
    <td>Transaction tag (exposed as a custom type with additional methods)
    </td>
  </tr>
  <tr>
    <td>timestamp</td>
    <td>&#10004;</td>
    <td><a href="#timestamp">Timestamp</a></td>
    <td>Transaction timestamp (exposed as a custom type with additional methods)
    </td>
  </tr>
  <tr>
    <td>currentIndex</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Transaction current index in the bundle</td>
  </tr>
  <tr>
    <td>lastIndex</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Transaction last index in the bundle</td>
  </tr>
  <tr>
    <td>bundle</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction bundle hash.</td>
  </tr>
  <tr>
    <td>trunkTransaction</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction trunk transaction.</td>
  </tr>
  <tr>
    <td>branchTransaction</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction branch transaction.</td>
  </tr>
  <tr>
    <td>nonce</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Transaction nonce.</td>
  </tr>
  <tr>
    <td>confirmed</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>Determines if the transaction is confirmed</td>
  </tr>
  <tr>
    <td>broadcasted?</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>Determines if the transaction was broadcasted to the network.
      </br>
      Will be true in the following scenarios:
      1. If the transaction was fetched from the network.
      <br />
      2. If the transaction was successfully broadcasted from the client itself.  
      Note: This property may only be required for clients with persistent storage.
    </td>
  </tr>
</table>

#### Storage Adapter Interface

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>get(key: string):<a href="#heading=h.fh3sa4qi8f48">Account</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets the account object for provided account name or id.</td>
  </tr>
  <tr>
    <td>getAll():
      <a href="#heading=h.fh3sa4qi8f48">Account</a>[]
    </td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all account objects from storage</td>
  </tr>
  <tr>
    <td>set(key: string, payload: string):void</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Stores account in storage.</td>
  </tr>
  <tr>
    <td>remove(key: string): void</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Removes account from storage.</td>
  </tr>
</table>

## Storage 

Multiple storage options should be used for managing data that requires persistence.  \
For wallet basic metadata for example user personal settings, theming options we could leverage a simple key-value [storage](https://capacitor.ionicframework.com/docs/apis/storage/).   
The key-value storage provided by [capacitor](https://capacitor.ionicframework.com/docs/apis/storage/) is not meant to be used for high-performance data storage applications. For transactions and address data management a separate data engine should be used, considering the fact that an account could have loads of transactions.  
For that purpose, a relational database such as [SQLite](https://github.com/jepiqueau/capacitor-sqlite) can be used. Following is an Entity Relationship (ERD) diagram that shows the logical representation of the data. An _account _is the basic entity in this database design. It has a one-to-many relationship with _addresses _i.e., an account could have multiple _addresses _but also an _address _could belong to only a single _account._ An _account _has a many-to-many relationship with _transactions _i.e., an _account _could have multiple _transactions _but it’s possible that a _transaction _belongs to multiple _accounts_. To accommodate for that, an additional table is added that stores account ids against transaction ids (hashes).  

Furthermore, a storage adapter is required by the rust layer because the storage operations (read/write) will be (mostly) done from that layer. A generic storage adapter is defined [here](https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cnmtyyhpbhe7).  


## Storage Adapter

The package should have a default opinionated storage mechanism but should also provide the ability for users to override the storage by specifying an adapter. As a default option, a relational database such as [SQLite](https://www.sqlite.org/index.html) can be used.  

See <a href="#storageAdapter">storage adapter</a> for adapter interface.

## Account

### API

#### Initialisation 

Initialises account
There could be the following scenarios in which an account can be initialised:



*   Mnemonic generated outside the stronghold:  In this case, the account should be initialised with mnemonic. It should communicate with the stronghold using its “importAccount” method and should expect an “id” in a response. 
*   Mnemonic generated inside the stronghold: In this case, the account should be initialised without mnemonic. It should communicate with the stronghold using its “createAccount” method and should expect an “id” in response.
*   Importing accounts from stronghold backup: In this case, the account should receive all initialisation properties from stronghold. Note that during backup, these properties should be passed to the stronghold so that it stores these configuration settings in the back up. See [ importAccounts()](#heading=h.vvkqbm8ffpkk)

_Following should be considered when initialising an account:_



*   An account should never be initialised directly. Instead, the only way an account could be initialized is through [AccountsManager.addAccount()](#heading=h.rmcbnl98kwxq) method.
*   An account should always be initialised after a successful response from the stronghold. If the stronghold fails to create an account, the account initialisation should error out. If the stronghold successfully creates an account, the account should be stored in the persistent storage. Upon successful store operation in the persistent storage, the user should be returned an account object.
*   If “provider” property is not passed, a random node should be selected from the “nodes” property.
*   If “type” property is not passed, “default” should be used as an account type.
*   “quorumSize” and “quorumThreshold” should be validated. For example, “quorumSize” should not be greater than the number of nodes provided by the user.
*   The “nodes” property should validate and remove duplicate node URLs.
*   All the properties of the returned account object should be read-only. They should not be allowed to be manipulated directly.


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>config</td>
   <td>&#10004;</td>
   <td><a href="#heading=h.o7ovb4caz69t">AccountConfig</a></td>
   <td>Initialisation method receives a configuration object.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>id</td>
   <td>string</td>
   <td colspan="3">First address on the seed. Required for referencing a seed in secure enclave/storage.</td>
  </tr>
  <tr>
   <td>alias</td>
   <td>string</td>
   <td colspan="3">Account name.</td>
  </tr>
  <tr>
   <td>createdAt</td>
   <td>Date</td>
   <td colspan="3">Account creation time.</td>
  </tr>
  <tr>
   <td>lastSyncedAt?</td>
   <td>Date</td>
   <td colspan="3">Time when the account was last synced with the tangle</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>Access modifiers</td>
   <td colspan="3">Public</td>
  </tr>
  <tr>
   <td>Errors</td>
   <td colspan="3">List of error messages</td>
  </tr>
  <tr>
   <td>Required client library methods</td>
   <td>None</td>
  </tr>
</table>



#### sync_addresses() 
Syncs addresses with the tangle. The method should ensure that the wallet local state has all used addresses plus an unused address. 
 
Following should be considered when implementing this method:

*   _The updated address history should not be written down in the database/persistent storage. Instead the method should only return the updated address history (with transaction hashes).  This will ensure that there are no partial writes to the database._
*   _To sync addresses for an account from scratch, index = 0 and gapLimit = 20 should be provided. _
*   _To sync addresses from the latest address, index = latest address index and gapLimit = 1 should be provided. 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>config</td>
   <td>&#10004;</td>
   <td><a href="#heading=h.o7ovb4caz69t">AccountConfig</a></td>
   <td>Initialisation method receives a configuration object.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>id</td>
   <td>string</td>
   <td colspan="3">First address on the seed. Required for referencing a seed in secure enclave/storage.</td>
  </tr>
  <tr>
   <td>alias</td>
   <td>string</td>
   <td colspan="3">Account name.</td>
  </tr>
  <tr>
   <td>createdAt</td>
   <td>Date</td>
   <td colspan="3">Account creation time.</td>
  </tr>
  <tr>
   <td>lastSyncedAt?</td>
   <td>Date</td>
   <td colspan="3">Time when the account was last synced with the tangle</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>Access modifiers</td>
   <td colspan="3">Public</td>
  </tr>
  <tr>
   <td>Errors</td>
   <td colspan="3">List of error messages</td>
  </tr>
  <tr>
   <td>Required client library methods</td>
   <td>
   <ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.24v5faxy5apt">getBalance()</a></li>
<li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a>
</li>
</ol>
</td>
  </tr>
</table>



#### sync_transactions() 

Syncs transactions with the tangle. The method should ensure that the wallet local state has transactions associated with the address history. 

Following should be considered when implementing this method:

*   The updated transaction history should not be written down in the database/persistent storage. Instead the method should only return the updated transaction history (with transaction hashes).
*   This method should check if there are any local transactions (with “broadcasted: false”) matching the transactions fetched from the network. If there are such transactions, their “broadcasted” property should be set to true.
*   For newly confirmed transactions, the method should ensure that it updates “confirmed” property of all its reattachments 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>hashes</td>
   <td>&#10004;</td>
   <td>string[]</td>
   <td>Transaction hashes. New transaction hashes should be calculated by running a difference of local transaction hashes with latest transaction hashes on the tangle. </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transactions</td>
   <td><a href="#transaction">findTransactions()</a></td>
   <td colspan="3">Transaction history</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>Access modifiers</td>
   <td colspan="3">Public</td>
  </tr>
  <tr>
   <td>Errors</td>
   <td colspan="3">List of error messages</td>
  </tr>
  <tr>
   <td>Required client library methods</td>
   <td>
   <ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
  </tr>
</table>

#### select_inputs() 

Selects input addresses for a value transaction.
Note: This method should only be used internally by [send()](#heading=h.fsosijwv1jo8). Also, the input selection method should ensure that the recipient address doesn’t match any of the selected inputs or the remainder address. 

See [Input Selection Process](#heading=h.eby2xfmp8y49) for implementation details


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>threshold</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Amount user wants to spend.</td>
  </tr>
  <tr>
   <td>address</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Recipient address.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>inputs</td>
   <td><a href="#heading=h.793tvqazsd6t">Address[]</a></td>
   <td colspan="3">Selected Inputs</td>
  </tr>
  <tr>
   <td>remainder</td>
   <td>Address</td>
   <td colspan="3">Remainder address object. Empty or null if there’s no need for a remainder</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>


#### send() 
Sends a value transaction to the tangle.  
Note: This method should only be exposed as a successful response from [sync()](#heading=h.e1ku5sowpkss). 
Following is the process for sending a value transaction:
*   _Ensure “amount” is not set to zero_
*   _Ensure “amount” does not exceed the total balance_ 
*   _Ensure “recipient address” has correct checksum_
*   _Validate “message” property semantics and size_
*   _Validate “tag”  property semantics and size. If it’s not provided, a default “tag” should be used_
*   _Select inputs by using [_selectInputs()](#heading=h.rmmpya16n7wa)_
*   _Pass transaction to stronghold for signing using its “signTransaction” method_
*   _Perform proof-of-work. “pow” property in the account object should determine if the proof of work should be offloaded. _
*   _Once proof-of-work is successfully performed, the transaction should be validated and stored in the persistent storage. _
*   _After persisting the transaction, it should be broadcasted to the network.  \
In case of broadcast error, there should be (three) attempts for automatic rebroadcasting. If all attempts fail, the send process should terminate and it should be left to the user to retry the failed transaction. For failed transactions, the “broadcasted” property in the transaction objects should be set to false. 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transfer</td>
   <td>&#10004;</td>
   <td><a href="#heading=h.e3zwawn7rxcw">Transfer</a></td>
   <td>Transfer object. </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transactions</td>
   <td><a href="#heading=h.mzpg65ps5g9y">Transaction[]</a></td>
   <td colspan="3">Newly made transaction.</td>
  </tr>
<tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### retry() 

Rebroadcasts failed transaction.
Note: This method should only be exposed as a successful response from [sync()](#heading=h.e1ku5sowpkss). 

Following is the process for retrying a failed transaction:

*   _Get transaction by using [getTransaction()](#heading=h.e0k18weql27y)_
*   Rebroadcast transaction
*   Update account in persistent storage 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>hash</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Transaction hash</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>transactions</td>
   <td><a href="#transaction">Transaction[]</a></td>
   <td>Newly made transaction.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### sync() 
Syncs account with the tangle. Account syncing process should ensure that the latest metadata (balance, transactions) associated with an account is fetched from the tangle and is stored locally.  
Note that this is a proposed design decision to enforce account syncing before every send. An alternative way would be to have the “send” method always exposed and internally ensuring that the account is synced before every proposed transaction. 

Following is the process for account syncing:_

*   _Sync addresses using [_syncAddresses()](#heading=h.5xu53aitrgh7)_
*   _Sync transactions using [ _syncTransactions()](#heading=h.y6ophiwcbuh7)_
*   _Store updated addresses and transactions information in persistent storage (If not explicitly set otherwise by the user). 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>index?</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Address index.  By default the length of addresses stored for this account should be used as an index.</td>
  </tr>
  <tr>
   <td>gapLimit?</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Number of address indexes that are generated.</td>
  </tr>
  <tr>
   <td>skipPersistence?</td>
   <td>&#10004;</td>
   <td>boolean</td>
   <td>Skips write to the database if set to true. 
This will be useful if a user wants to scan the tangle for further addresses to find balance.  
See <a href="https://docs.iota.org/docs/wallets/0.1/trinity/how-to-guides/perform-a-snapshot-transition">snapshot transition</a> feature provided by Trinity wallet.
   </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>depositAddress</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>
   </td>
   <td colspan="3">Deposit address. Only exposed on successful completion of account syncing process.</td>
  </tr>
  <tr>
   <td><a href="#heading=h.fsosijwv1jo8">send: _send</a></td>
   <td>function</td>
   <td colspan="3">Send transaction method. Only exposed on successful completion of account syncing process.</td>
  </tr>
  <tr>
   <td><a href="#heading=h.2mx12pal8zwc">retry: _retry</a></td>
   <td>function</td>
   <td colspan="3">Retry transactions method. Rebroadcasts failed transaction. Only exposed on successful completion of account syncing process.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>

####  reattach() 
Reattaches unconfirmed transaction to the tangle. 
Following should be considered when implementing this method:_

*   _Only an unconfirmed transaction should be allowed to reattach. The method should validate the confirmation state of the provided transaction. If a transaction hash of a confirmed transaction is provided, the method should error out. _
*   _The method should also validate if the transaction reattachment is necessary. This can be done by checking if the transaction falls below max depth. The criteria of checking whether the transaction has fallen below max depth is through time. If 11 minutes have passed since the timestamp of the most recent (reattachment), the transaction can be allowed to be reattached. See [this ](https://github.com/iotaledger/trinity-wallet/blob/3fab4f671c97e805a2b0ade99b4abb8b508c2842/src/shared/libs/iota/transfers.js#L141)implementation for reference. _
*   _Once reattached, the transaction should be stored in the persistent storage. _
*   _If the transaction was reattached via polling, an event should be emitted via [reattachment event](#heading=h.4sgn6u46no39) to notify all subscribers about this reattachment. 

<table>
  <tr>
   <td colspan="3"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>hash</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Transaction hash.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transaction</td>
   <td><a href="#heading=h.mzpg65ps5g9y">Transaction[]</a></td>
   <td colspan="3">Newly reattached transaction.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>

#### sendMessage() 
Sends a zero value transaction to the tangle

Following is the process for sending a zero value message:_

*   _Ensure “amount” is set to zero_
*   _Ensure “recipient address” has correct checksum_
*   _Validate “message” property semantics and size_
*   _Validate “tag”  property semantics and size. If it’s not provided, a default “tag” should be used_
*   _On successful broadcast of the zero value transaction, the new transaction should be stored in the persistent storage and its “broadcasted” property should be set to true. _

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>message</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Message to send to the tangle.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transaction</td>
   <td><a href="#heading=h.mzpg65ps5g9y">Transaction[]</a></td>
   <td colspan="3">Newly broadcasted transaction.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### totalBalance() 
Gets total account balance

_Total balance should directly be read from the local storage. To read the latest account balance from the network, [sync()](#heading=h.e1ku5sowpkss) should be used first. 

<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.pgrdghlc7z4b">Value</a>
   </td>
   <td>Account total balance.</td>
  </tr>
  <tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### availableBalance() 
Gets available account balance. Available account balance is the balance users are allowed to spend. It should subtract the already used balance from the total balance. 

For example, if a user with 50i total account balance has made a transaction spending 30i, the available balance should be (50i-30i) 20i._

_Available balance should directly be read from the local storage. To read the latest account balance from the network, [sync()](#heading=h.e1ku5sowpkss) should be used first.

<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Type</strong>
   </td>
   <td colspan="3"><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.pgrdghlc7z4b">Value</a>
   </td>
   <td>Account available balance.
   </td>
  </tr>
<tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>

#### setAlias() 

Updates account name

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>alias</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>New account name.</td>
  </tr>
<tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>


#### listTransactions() 
Gets transactions.
Transactions should be directly read from the local storage. To ensure the local database is updated with the latest transactions, [sync()](#heading=h.e1ku5sowpkss) should be used first.


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>count</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Number of (most recent) transactions</td>
  </tr>
  <tr>
   <td>from</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Subset of transactions. For example: count = 10, from=5 - it should return ten transactions skipping the most recent 5 transactions.
   </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong>
   </td>
  </tr>
  <tr>
  <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transactions</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
   </td>
   <td>All transactions.
   </td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### listReceivedTransactions() 
Gets all received transactions.

Transactions should be directly read from the local storage. To ensure the local database is updated with the latest transactions, [sync()](#heading=h.e1ku5sowpkss) should be used first. 

An alternate way would be to add a “type” (“received” | “sent”) parameter to the [listTransactions() ](#heading=h.c8phd7cqv0zc)method. 


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>count</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Number of (most recent) received transactions</td>
  </tr>
  <tr>
   <td>from</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Subset of received transactions.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
	<td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transactions</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
   </td>
   <td colspan="3">All received transactions.
   </td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### listSentTransactions() 
Gets all sent transactions

_Transactions should be directly read from the local storage. To ensure the local database is updated with the latest transactions, [sync()](#heading=h.e1ku5sowpkss) should be used first. _

_An alternate way would be to add a “type” (“received” | “sent”) parameter to the [listTransactions() ](#heading=h.c8phd7cqv0zc)method. _


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>count</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Number of (most recent) sent transactions</td>
  </tr>
  <tr>
   <td>from</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Subset of sent transactions.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
	<td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transactions</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
   </td>
   <td colspan="3">All sent transactions.</td>
  </tr>
</table>

#### listFailedTransactions() 
_Gets all failed (broadcasted property set as false) transactions._

_Transactions should be directly read from the local storage. _

<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
  <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transactions</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
   </td>
   <td>All failed transactions.
   </td>
  </tr>
  <tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### listUnconfirmedTransactions() 
Gets all unconfirmed (confirmed property set as false) transactions.
_Transactions should be directly read from the local storage.  

<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
  <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transactions</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>[]
   </td>
   <td>All unconfirmed transactions.
   </td>
  </tr>
<tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### getTransaction() 
Gets transaction for provided hash.

_Transaction objects should be directly read from the local storage. To ensure the local database is updated with the latest transactions, [sync()](#heading=h.e1ku5sowpkss) should be used first. 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>hash</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Transaction hash</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
  <td>transaction</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.cn1ufiumug7n">Transaction</a>
   </td>
   <td>Transaction object.
   </td>
  </tr>
<tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### listAddresses() 
Gets all addresses
<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>[]
   </td>
   <td>All addresses.</td>
  </tr>
<tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### listUnspent() 
Gets all unspent input addresses
<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>[]
   </td>
   <td>All unspent input addresses.</td>
  </tr>
  <tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>

#### generateAddress() 
Gets a new unused address.

<table>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>
   </td>
   <td>A new address object.
   </td>
  </tr>
  <tr>
<td colspan="3"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>


## Accounts Manager

An accounts manager class should be publicly available for users to use. Only using an accounts manager, the users should be able to create, update, delete or manage multiple accounts. The implementation details of a specific account should be abstracted away from the users using this accounts manager wrapper. 

### API


#### Initialisation - _Initialises accounts manager_

_Accounts manager initialisation should validate the adapter object semantics and should return an instance of the accounts manager._


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>adapter?</td>
   <td>&#10004;</td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.wwv1rlv0cmx5">Adapter</a>
   </td>
   <td>Initialisation method receives an optional storage adapter.</td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>accounts</td>
   <td>
<a href="#heading=h.fh3sa4qi8f48">Account</a>[]
   </td>
   <td colspan="3">Persisted accounts.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### addAccount() 
Adds new account

_See account [initialisation](#heading=h.o7ovb4caz69t) for detailed implementation guidelines_

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>config</td>
   <td>&#10004;</td>
   <td>
<a href="#heading=h.o7ovb4caz69t">AccountConfig</a>
   </td>
   <td>Account configuration object.
   </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>accounts</td>
   <td>
<a href="#heading=h.fh3sa4qi8f48">Account</a>[]
   </td>
   <td colspan="3">Persisted accounts.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### removeAccount() 
Removes account 
Following should be considered when removing an account:
*   _An account should first be removed from the stronghold using its “removeAccount” method._
*   _Once the account references have been removed from the stronghold, the account should be deleted from the persistent storage. _

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>identifier</td>
   <td>&#10004;</td>
   <td>{ address: &lt;string> } |  
{ alias: &lt;string>  } |
<p>
{ id: &lt;number> } |
<p>
{ index: &lt;number }
   </td>
   <td>Identifier. Could be one of address, alias, id or index. 
   </td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### syncAccounts() 
Syncs all stored accounts with the tangle. Syncing should get the latest balance for all accounts, find any new transactions associated with the stored account. \
See [Accounts Syncing Process](#heading=h.tamnlkuh0nce)


<table>
  <tr>
   <td colspan="3" ><strong>Returns</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>depositAddress
   </td>
   <td><a href="https://docs.google.com/document/d/17JHw7HpNn3_qKKXaxoQJFxQv4em9xomh0EvvWOzIQzI/edit#heading=h.5s4azzg1mfxt">Address</a>
   </td>
   <td>Deposit address. Only exposed on successful completion of account syncing process.
   </td>
  </tr>
  <tr>
   <td>send: 
<a href="#heading=h.fsosijwv1jo8">_send</a>
   </td>
   <td>function
   </td>
   <td>Send transaction method. Only exposed on successful completion of account syncing process.
   </td>
  </tr>
  <tr>
   <td>retry: 
   <a href="#heading=h.2mx12pal8zwc">_retry</a>
   </td>
   <td>function
   </td>
   <td>Retry transactions method. Rebroadcasts failed transaction. Only exposed on successful completion of account syncing process.
   </td>
  </tr>
    <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### move() 
Initiates an internal transaction between accounts. This method should leverage the [send() ](#heading=h.fsosijwv1jo8)method from the sender account and should initiate a transaction to the receiver account.


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>from</td>
   <td>&#10004;</td>
   <td>{ address: &lt;string> } |  \
{ alias: &lt;string>  } |
<p>
{ id: &lt;number> } |
<p>
{ index: &lt;number }
   </td>
   <td>Identifier. Could be one of address, alias, id or index. 
   </td>
  </tr>
  <tr>
   <td>to</td>
   <td>&#10004;</td>
   <td>{ address: &lt;string> } |  \
{ alias: &lt;string>  } |
<p>
{ id: &lt;number> } |
<p>
{ index: &lt;number }
   </td>
   <td>Identifier. Could be one of address, alias, id or index. 
   </td>
  </tr>
  <tr>
   <td>amount</td>
   <td>&#10004;</td>
   <td>number</td>
   <td>Transaction amount</td>
  </tr>
  <tr>	
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>

#### backup() 
Safely creates a backup of the accounts to destination._ _The file could simply be in a JSON format containing the address & transaction histories for accounts.  \
This method should provide the stronghold instance with metadata of all accounts. 


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>destination</td>
   <td>&#10004;</td>
   <td>string</td>
   <td>Path where the backup should be stored.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### importAccounts
Import (backed up) accounts 

Implementation details are not finalised.

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>accounts</td>
   <td>&#10004;</td>
<td><a href="#heading=h.fh3sa4qi8f48">Account</a>[]
   </td>
   <td>Account object.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### getAccount() 
Returns the account associated with the provided address.


<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>identifier</td>
   <td>&#10004;</td>
   <td>{ address: &lt;string> } |  \
{ alias: &lt;string>  } |
<p>
{ id: &lt;number> } |
<p>
{ index: &lt;number }
   </td>
   <td>Identifier. Could be one of address, alias, id or index. 
   </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong>
   </td>
   <td><strong>Type</strong>
   </td>
   <td colspan="3"><strong>Description</strong>
   </td>
  </tr>
  <tr>
   <td>Account object
   </td>
   <td>
<a href="#heading=h.fh3sa4qi8f48">Account</a>
   </td>
   <td colspan="3">Account associated with identifier
   </td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



#### reattach() 
Reattaches an unconfirmed transaction.

See[reattach()](#heading=h.wpqun6wk6psu) method on an account object for implementation details. This method is a wrapper method provided for convenience. A user could directly access the [reattach()](#heading=h.wpqun6wk6psu) method on an account object. 

<table>
  <tr>
   <td colspan="4"><strong>Parameters</strong></td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Required</strong></td>
   <td><strong>Type</strong></td>
   <td><strong>Description</strong></td>
  </tr>
  <tr>
   <td>identifier</td>
   <td>&#10004;</td>
   <td>{ address: &lt;string> } |  \
{ alias: &lt;string>  } |
<p>
{ id: &lt;number> } |
<p>
{ index: &lt;number }
   </td>
   <td>Identifier. Could be one of address, alias, id or index. 
   </td>
  </tr>
  <tr>
   <td>hash</td>
   <td>&#10004;</td>
   <td>string
   </td>
   <td>Transaction hash
   </td>
  </tr>
  <tr>
   <td colspan="4"><strong>Returns</strong>
   </td>
  </tr>
  <tr>
   <td><strong>Name</strong></td>
   <td><strong>Type</strong></td>
   <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
   <td>transaction
   </td>
   <td>
<a href="#heading=h.mzpg65ps5g9y">Transaction[]</a>
   </td>
   <td colspan="3">Newly reattached transaction</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages</td>
</tr>
<tr>
<td>Required client library methods</td>
<td>
<ol><li><a href="https://docs.google.com/document/d/1mH0_mjlPv5jZZWFEe20BTzVzXJ6XEXOqtY7jkvNHyiY/edit#heading=h.eoox82z3y6rj">findTransactions()</a></li>
</ol>
</td>
</tr>
</table>



## Events 
Events are categorised as following:

1. Reactive messages emitted from the node software whenever the state on the node changes. For example, emitting new transaction data received by the node. Clients (Wallet) can subscribe to these events to get notified if any relevant change occurs on the node. See [example](https://github.com/iotaledger/wallet-spec/tree/events).
2. Messages emitted from the wallet library whenever there are any important state changes. Note that in cases, where a user triggered action leads to a state change, the messages would not be emitted. For example, if a user explicitly triggers a [sync(](#heading=h.e1ku5sowpkss)) action leading to a state change, an explicit emission of messages through events is not necessary.

Following are the events description for **category 1**. On every update sent from the node software via an event, the wallet library should update internal (persistent) storage and should also emit events via **category 2**. 


#### Monitor address for balance changes


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >&lt;Address:Balance>
   </td>
   <td>Index 1: Address  
Index 2: New Balance on the address 
   </td>
  </tr>
</table>



#### Monitor address for new transactions 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >&lt;Address:Transaction>
   </td>
   <td>Index 1: Address  \
Index 2: Transaction hash of the new transaction  \

   </td>
  </tr>
</table>



#### Monitor transaction for confirmation state 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >&lt;TransactionHash>
   </td>
   <td>Index 1: Transaction Hash  \
Index 2: Confirmation state \

   </td>
  </tr>
</table>


Following are the events description for **category 2**. They could be triggered via events from **category 1** or through [polling](#heading=h.6okotyh7cn9v). 


#### Monitor for balance changes


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >balances
   </td>
   <td>[{ accountId, address, balance }]
   </td>
  </tr>
</table>



#### Monitor for new transactions 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >transactions
   </td>
   <td>[{ accountId, transactions }]
   </td>
  </tr>
</table>



#### Monitor for confirmation state 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >confirmations
   </td>
   <td>[{ accountId, transactions  }] \

   </td>
  </tr>
</table>



#### Monitor for reattachments 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >reattachments
   </td>
   <td>[{ accountId, transactions  }] \

   </td>
  </tr>
</table>



#### Monitor for broadcasts 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >broadcasts
   </td>
   <td>[{ accountId, transactions  }] \

   </td>
  </tr>
</table>



#### Monitor for errors 


<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3" >error
   </td>
   <td>{ type, message  } \

   </td>
  </tr>
</table>



## Privacy


To maintain the financial privacy of the wallet users, the application/wallet should enforce strategies that will guarantee a certain level of anonymity to the user. Following strategies should be followed:



1. The wallet should only use a single address per transaction i.e., if an address is already used in a transaction, it should not be used as a deposit address and instead a new address should be generated.
2. If (accidentally), funds arrive at a spent address, the wallet should do an internal sweep before allowing the funds to be spent. 
3. The input selection strategy should expose as little information as possible. See input selection for details.

Some other privacy enhancing techniques can be found [here](https://docs.google.com/document/d/1frk4r1Eq4hnGGOiKWkDiGTK5QQxKbfrvl7Iol7OZ-dc/edit#). 


## Input Selection


The goal of input selection in the application/wallet should be to avoid change/remainder. The change output leaves a clue to the user's future spends. There should be a standardised input selection strategy used by the wallet. The steps for input selection are as follows:



1. Try to select an input with an exact match. For example, if a user intends to spend _X_ iotas, the wallet should do a search on addresses and should try to find an address that has _X _iotas as available balance.
2. If the previous step fails, try to select a combination of inputs that satisfy the amount leaving no change. For example, consider a scenario where the wallet with account name _Foo _has three addresses _A_, _B _and _C _with _10, 20 _and_ 50 _balances respectively. If a user intends to spend _X=30_ iotas, the application should search if there’s an exact match (step no. 1). Clearly, in this case, no address balance matches _X, _therefore, the wallet should search for a subset of addresses that accumulates their balances to _X._ In this scenario, it should be _A _and _B._
3. If both the previous steps fail, the wallet should create a combination of inputs that reveal the minimum change. 

 \
Reference implementation of different input selection algorithms for Bitcoin can be found [here](https://github.com/bitcoinjs/coinselect). \
 \
 Also, the implementation of step no. 2 is quite similar to the [subset sum problem](https://en.wikipedia.org/wiki/Subset_sum_problem). Given a _total _and a set of non-negative numbers (_inputs_), we need to determine if there is a subset which adds up to the _total_.


## Accounts Syncing Process


The account syncing process should detect all (used) accounts on a seed with their corresponding address and transaction history. Once, all accounts with their histories are detected, the wallet should accumulate total balance. The syncing process should work as follows: 

1. Start with the account at index 0, generate [gap limit](https://blog.blockonomics.co/bitcoin-what-is-this-gap-limit-4f098e52d7e1) number of addresses, default to 20
2. Check for transactions and balances on the generated addresses.
3. If there are no transactions and zero balances on all addresses, the process for generating addresses and finding transactions and balances should be stopped. 
4. If there are transactions or any address has balance, generate more gap limit number of addresses starting from the index of the last address with transactions or balance. 
5. Steps (1-4) should also be done for account at index 1. The general idea is that _n + 1_ accounts should be checked if account _n _has transactions or any balance.

Treat accounts like addresses. Only allow 1 latest unused account.

Scenario 1: Wallet transaction and address history stored in Stronghold backup



*   Start syncing from the latest address index stored in the Stronghold backup
*   Also provide a “Full sync” function to resync from index 0 across all accounts
*   Also provide “Find more history” function sync a further 50 addresses

Scenario 2: User has no backup file



*   Start syncing from account 0 address 0


## Polling

 A background process automatically performing several tasks periodically should be developed to be part of the wallet library. The goal of the background process is to perform the following tasks:  


*   **Sync accounts:** The background process should sync all accounts with the network. This should be done using [manager.syncAccounts() ](#heading=h.t6h98wic1xp6)method. \
If new transactions are detected, a [transactions ](#heading=h.y2yqq8oe4yd9)event should be used to notify all subscribers. \
If new balances are detected, a [balances](#heading=h.hu9n3emr34m2) event should be used to notify all subscribers. \
If new confirmations are detected, a [confirmations](#heading=h.3o9qzrm36uj) event should be used to notify all subscribers. 
*   **Retry failed transactions:** The background process should check if there are any transactions that failed to broadcast to the network. On successful broadcast, an event should be [emitted](#heading=h.r857yk2r7k4h) to all subscribers. To list failed transactions, [listFailedTransaction()](#heading=h.zhr8uh37dyyv) should be used.  \
Note that if there are multiple failed transactions, the priority should be given to the old ones. 
*   **Reattach:** The background process should check if there are any (unconfirmed) transactions that require reattachments. The detailed implementation flow for reattachment can be found [here](#heading=h.tejv06xx6ssr). 

Following should be considered for implementation:

*   Invoking a task explicitly while polling is performing it should lead to an error. For example, if the polling process is already syncing accounts and a user explicitly calls account.sync(), it should throw an error.
*   Errors during the polling process should be communicated to subscribers via error event.

Ideally, the background process should have a recurring checker that is sequentially performing all the above mentioned tasks. The implementation should ensure that future tasks can be easily added to the background process. For reference, see Trinity’s [implementation](https://github.com/iotaledger/trinity-wallet/blob/develop/src/mobile/src/ui/components/Poll.js) of the poll component. 
