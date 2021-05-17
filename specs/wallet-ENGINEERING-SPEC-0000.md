# Wallet Library Spec

## Table of Contents <!-- omit in toc -->
- [Wallet Library Spec](#wallet-library-spec)
  - [Introduction](#introduction)
  - [Considerations](#considerations)
  - [Naming Conventions](#naming-conventions)
  - [Interfaces](#interfaces)
      - [AccountConfiguration](#accountconfiguration)
      - [AccountObject](#accountobject)
      - [SyncedAccountObject](#syncedaccountobject)
      - [AccountManagerObject](#accountmanagerobject)
      - [Address](#address)
      - [Node](#node)
      - [Timestamp](#timestamp)
      - [Transfer](#transfer)
      - [Value](#value)
      - [Input](#input)
      - [OutputAddress](#outputaddress)
      - [Output](#output)
      - [UnsignedDataPayload](#unsigneddatapayload)
      - [SignedDataPayload](#signeddatapayload)
      - [IndexationPayload](#indexationpayload)
      - [UnsignedTransaction](#unsignedtransaction)
      - [Ed25519Signature](#ed25519signature)
      - [SignatureUnblockBlock](#signatureunblockblock)
      - [ReferenceUnblockBlock](#referenceunblockblock)
      - [SignedTransactionPayload](#signedtransactionpayload)
      - [Message](#message)
      - [StorageAdapter](#storageadapter)
  - [Storage](#storage)
  - [Storage Adapter](#storage-adapter)
  - [Account](#account)
    - [API](#api)
      - [Initialisation](#initialisation)
      - [sync_addresses()](#sync_addresses)
      - [sync_messages()](#sync_messages)
      - [select_inputs()](#select_inputs)
      - [send()](#send)
      - [retry()](#retry)
      - [sync()](#sync)
      - [reattach()](#reattach)
      - [total_balance()](#total_balance)
      - [available_balance()](#available_balance)
      - [set_alias()](#set_alias)
      - [list_messages()](#list_messages)
      - [list_received_messages()](#list_received_messages)
      - [list_sent_messages()](#list_sent_messages)
      - [list_failed_messages()](#list_failed_messages)
      - [list_unconfirmed_messages()](#list_unconfirmed_messages)
      - [get_message()](#get_message)
      - [list_addresses()](#list_addresses)
      - [list_unspent_addresses()](#list_unspent_addresses)
      - [generate_address()](#generate_address)
  - [Account Manager](#account-manager)
    - [API](#api-1)
      - [Initialisation](#initialisation-1)
      - [add_account()](#add_account)
      - [remove_account()](#remove_account)
      - [sync_accounts()](#sync_accounts)
      - [move()](#move)
      - [backup()](#backup)
      - [import_accounts](#import_accounts)
      - [get_account()](#get_account)
      - [reattach()](#reattach-1)
  - [Events](#events)
    - [Category 1 events](#category-1-events)
      - [Monitor address for balance changes](#monitor-address-for-balance-changes)
      - [Monitor address for new messages](#monitor-address-for-new-messages)
      - [Monitor message for confirmation state](#monitor-message-for-confirmation-state)
    - [Category 2 events](#category-2-events)
      - [Monitor for balance changes](#monitor-for-balance-changes)
      - [Monitor for new messages](#monitor-for-new-messages)
      - [Monitor for confirmation state](#monitor-for-confirmation-state)
      - [Monitor for reattachments](#monitor-for-reattachments)
      - [Monitor for broadcasts](#monitor-for-broadcasts)
      - [Monitor for errors](#monitor-for-errors)
  - [Privacy](#privacy)
  - [Input Selection](#input-selection)
  - [Account Syncing Process](#account-syncing-process)
  - [Polling](#polling)

## Introduction

The wallet library is a stateful package with a standardised interface to build applications with IOTA value transactions. The package will be compatible with different platforms such as web, desktop and mobile. 

The package introduces the concept of an _account_. An account is a reference or a label to a [seed](https://docs.iota.org/docs/getting-started/0.1/clients/seeds). An account has certain properties such as [addresses](https://github.com/Wollac/protocol-rfcs/blob/bech32-address-format/text/0020-bech32-address-format/0020-bech32-address-format.md) and [messages](https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md). An account has various possible behaviours, including moving funds, looking for new messages, and making copies of message histories. An account should also be able to provide a degree of financial privacy and this should not incur any overhead. 

A similar [package](https://docs.iota.org/docs/client-libraries/0.1/account-module/introduction/overview) was previously developed but this becomes obsolete with the introduction of Ed25519 signatures. The previous account package was limited to a single account. As an improvement, the (new) package will be able to manage multiple accounts. 

To summarize, the main motivation behind this package is to offer a simplified (stateful) approach to handle IOTA payments.

## Considerations

*   Seeds should be stored and managed separately in a secure enclave and should never leave the secure environment. Secure enclaves include software enclaves such as IOTA’s Rust-based Stronghold library or hardware enclaves such as a Ledger Nano;
*   The secure enclave should have the ability to generate addresses and sign messages upon receipt of a message, and return the output in a message. If the secure enclave is initialised with a pre-generated seed, the sender process should immediately remove the seed traces from memory. 

## Naming Conventions

The primary language is [Rust](https://github.com/rust-lang/rust). Therefore, standard Rust naming [conventions](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html) are followed. All interfaces (types) use _CamelCase_ while all function and variable names use _snake\_case_.

## Interfaces

#### AccountConfiguration

Account configuration or initialization object. It should support parameters accepted by high level [client](https://github.com/iotaledger/iota.rs) libraries.

<table>
  <tr>
    <td>
      <strong>Property</strong>
    </td>
    <td>
      <strong>Required</strong>
    </td>
    <td>
      <strong>Type</strong>
    </td>
    <td>
      <strong>Description</strong>
    </td>
  </tr>
  <tr>
    <td>seed</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>BIP-39 mnemonic. When importing an account from Stronghold backup, the seed will not be required.</td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>SHA-256 hash of the first address on the seed (m/44'/0'/0'/0'/0'). Required for referencing a seed in Stronghold. The id should be provided by Stronghold.</td>
  </tr>
  <tr>
    <td>index</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Account index in <a href="https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki">BIP-44</a> derivation path.</td>
  </tr>
  <tr>
    <td>alias</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Account name. If not provided, <code>Account + ${index}</code> should be used. When importing an account from Stronghold backup, the alias will be required from Stronghold.</td>
  </tr>
  <tr>
    <td>pow</td>
    <td>&#10008;</td>
    <td>‘local’ | ‘remote’</td>
    <td>
    Proof of work settings. Defaults to ‘local’. 
      <ul>
        <li>‘local’: Should be performed on device;</li>
        <li>‘remote’: Should be performed on the node.</li>
      </ul>
    </td>
  </tr>
  <tr>
    <td>nodes</td>
    <td>&#10004;</td>
    <td><a href="#node">node</a>[]</td>
    <td>A list of nodes to connect to.</td>
  </tr>
  <tr>
    <td>quorum_size</td>
    <td>&#10008;</td>
    <td>number</td>
    <td>If multiple nodes are provided, quorum size determines the number of nodes to query to check for quorum.</td>
  </tr>
  <tr>
    <td>quorum_threshold</td>
    <td>&#10008;</td>
    <td>number</td>
    <td>Minimum number of nodes from the quorum pool that need to agree to consider a result true.</td>
  </tr>
  <tr>
    <td>network</td>
    <td>&#10008;</td>
    <td>‘mainnet’ | ‘devnet’ | ‘comnet’</td>
    <td>IOTA public network.</td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10008;</td>
    <td>‘default’ | ‘ledger’</td>
    <td>Account type. Required for differentiating ledger vs non-ledger accounts.</td>
  </tr>
  <tr>
    <td>provider</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Node URL.</td>
  </tr>
  <tr>
    <td>created_at</td>
    <td>&#10008;</td>
    <td>Date</td>
    <td>Time of account creation.</td>
  </tr>
  <tr>
    <td>messages</td>
    <td>&#10008;</td>
    <td><a href="#message">Message</a>[]</td>
    <td>Messages associated with account. Accounts can be initialised with locally stored messages.</td>
  </tr>
  <tr>
    <td>addresses</td>
    <td>&#10008;</td>
    <td><a href="#address">Address</a>[]</td>
    <td>Address history associated with the account. Accounts can be initialised with locally stored address history.</td>
  </tr>
</table>

#### AccountObject

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>SHA-256 hash of the first address on the seed (m/44'/0'/0'/0/0). Required for referencing a seed in Stronghold.</td>
  </tr>
  <tr>
    <td>alias</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Account name.</td>
  </tr>
  <tr>
    <td>created_at</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Account creation time.</td>
  </tr>
  <tr>
    <td>last_synced_at</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Time the account was last synced with the Tangle.</td>
  </tr>
  <tr>
    <td><a href="#sync">sync()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Syncs account with the Tangle.</td>
  </tr>
  <tr>
    <td><a href="#reattach">reattach()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Reattaches unconfirmed transaction to the Tangle.</td>
  </tr>
  <tr>
    <td><a href="#totalbalance">total_balance()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets total account balance.</td>
  </tr>
  <tr>
    <td><a href="#availablebalance">available_balance()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets available account balance.</td>
  </tr>
  <tr>
    <td><a href="#setalias">set_alias()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Updates account name.</td>
  </tr>
  <tr>
    <td><a href="#listmessages">list_messages()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets messages.</td>
  </tr>
  <tr>
    <td><a href="#listreceivedmessages">list_received_messages()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all received messages.</td>
  </tr>
  <tr>
    <td><a href="#listsentmessages">list_sent_messages()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all sent messages.</td>
  </tr>
  <tr>
    <td><a href="#listfailedmessages">list_failed_messages()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all failed messages.</td>
  </tr>
  <tr>
    <td><a href="#listunconfirmedmessages">list_unconfirmed_messages()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all unconfirmed messages.</td>
  </tr>
  <tr>
    <td><a href="#getmessage">get_message()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets message for provided id.</td>
  </tr>
  <tr>
    <td><a href="#listaddresses">list_addresses()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all addresses.</td>
  </tr>
  <tr>
    <td><a href="#listunspent">list_unspent_addresses()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all unspent input addresses.</td>
  </tr>
  <tr>
    <td><a href="#generateaddress">generate_address()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets the latest unused address.</td>
  </tr>
</table>

#### SyncedAccountObject

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>deposit_address</td>
    <td>&#10004;</td>
    <td><a href="#address">Address</a></td>
    <td>Deposit address. Only exposed on successful completion of account syncing process.</td>
  </tr>
  <tr>
    <td><a href="#send">send()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Send transaction method. Only exposed on successful completion of account syncing process.</td>
  </tr>
  <tr>
    <td><a href="#retry">retry()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Rebroadcasts failed transaction. Only exposed on successful completion of account syncing process.</td>
  </tr>
</table>

#### AccountManagerObject

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>accounts</td>
    <td>&#10004;</td>
    <td><a href="#account">Account</a>[]</td>
    <td>Account objects.</td>
  </tr>
  <tr>
    <td><a href="#addaccount">add_account()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Adds a new account.</td>
  </tr>
  <tr>
    <td><a href="#removeaccount">remove_account()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Removes an account.</td>
  </tr>
  <tr>
    <td><a href="#syncaccounts">sync_accounts()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Syncs all stored accounts with the Tangle.</td>
  </tr>
  <tr>
    <td><a href="#move">move()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Inititates an internal transaction between accounts.</td>
  </tr>
  <tr>
    <td><a href="#backup">backup()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Creates a backup to a provided destination.</td>
  </tr>
  <tr>
    <td><a href="#importaccounts">import_accounts()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Imports backed up accounts.</td>
  </tr>
  <tr>
    <td><a href="#getaccount">get_account()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Returns the account associated with the provided address.</td>
  </tr>
  <tr>
    <td><a href="#reattach">reattach()</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Reattaches an unconfirmed transaction.</td>
  </tr>
</table>

#### Address 

Useful [reference](https://medium.com/@harshagoli/hd-wallets-explained-from-high-level-to-nuts-and-bolts-9a41545f5b0) for address management in Hierarchical Deterministic (HD) wallets.

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Address <a href="https://github.com/Wollac/protocol-rfcs/blob/bech32-address-format/text/0020-bech32-address-format/0020-bech32-address-format.md">(Bech32)</a> string.</td>
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
    <td>internal</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>Determines if an address is a public or an internal (change) address. See the concept of <a href="https://medium.com/@harshagoli/hd-wallets-explained-from-high-level-to-nuts-and-bolts-9a41545f5b0">chain node</a> for more details.</td>
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
    <td><strong>Required</strong></td>
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
    <td>username</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Node username. Only required if node requires authorisation.</td>
  </tr>
  <tr>
    <td>password</td>
    <td>&#10008;</td>
    <td>string</td>
    <td>Node password. Only required if node requires authorisation.</td>
  </tr>
  <tr>
    <td>network</td>
    <td>&#10004;</td>
    <td>‘mainnet’ | ‘devnet’ | ‘comnet’</td>
    <td>IOTA public network name.</td>
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
    <td>Transaction timestamp in various formats. For example: MM-DD-YYYY, DD MM YYYY hh:mm:ss.</td>
  </tr>
</table>

#### Transfer 

Transfer object required for creating a transaction. It allows end-users to specify the transaction amount and recipient address.

Note: Currently, it is not possible to send multiple payloads as part of the message. That is why tag property is omitted from this interface. See [this](https://github.com/iotaledger/protocol-rfcs/pull/18#discussion_r468432794) for details.

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
    <td>indexation_key</td>
    <td>&#10008;</td>
    <td><a href="#indexationpayload">Indexation Payload</a></td>
    <td>(Optional) Indexation payload.</td>
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
    <td>with_denomination():string</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction amount with unit.</td>
  </tr>
  <tr>
    <td>without_denomination():number</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Transaction amount without unit.</td>
  </tr>
</table>

#### Input

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Input type. Defaults to <code>0</code>.</td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>BLAKE2b-256 hash of the transaction.</td>
  </tr>
  <tr>
    <td>output_index</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Index of the output on the referenced transaction.</td>
  </tr>
</table>

#### OutputAddress

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to value <code>0</code> to denote an Ed25519 address.</td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>If type is set to <code>0</code>, it should contain an Ed25519 address.</td>
  </tr>
</table>

#### Output

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Output type. Defaults to <code>0</code>.</td>
  </tr>
  <tr>
    <td>address</td>
    <td>&#10004;</td>
    <td><a href="#outputaddress">OutputAddress</a></td>
    <td>Output address.</td>
  </tr>
  <tr>
    <td>amount</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Amount of tokens to deposit.</td>
  </tr>
</table>

#### UnsignedDataPayload

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to <code>2</code> to denote a unsigned data payload.</td>
  </tr>
  <tr>
    <td>data</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Data of unsigned payload.</td>
  </tr>
</table>

#### SignedDataPayload

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to <code>3</code> to denote a signed data payload.</td>
  </tr>
  <tr>
    <td>data</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Data of signed data payload.</td>
  </tr>
  <tr>
    <td>public_key</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Ed25519 public key used to verify the signature.</td>
  </tr>
  <tr>
    <td>signature</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Signature of signing data.</td>
  </tr>
</table>

#### IndexationPayload

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>index</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Indexation key.</td>
  </tr>
  <tr>
    <td>data</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Indexation data.</td>
  </tr>
</table>

#### UnsignedTransaction

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Transaction type. Defaults to <code>0</code>.</td>
  </tr>
  <tr>
    <td>inputs_count</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Amount of inputs proceeding.</td>
  </tr>
  <tr>
    <td>inputs</td>
    <td>&#10004;</td>
    <td><a href="#input">Input</a>[]</td>
    <td>Transaction inputs.</td>
  </tr>
  <tr>
    <td>outputs_count</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Amount of outputs proceeding.</td>
  </tr>
  <tr>
    <td>outputs</td>
    <td>&#10004;</td>
    <td><a href="#outputs">Output</a>[]</td>
    <td>Output address.</td>
  </tr>
  <tr>
    <td>payload_length</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Length of optional payload.</td>
  </tr>
  <tr>
    <td>payload</td>
    <td>&#10004;</td>
    <td>
    <a href="#unsigneddatapayload">UnsignedDataPayload</a> | 
    <a href="#signeddatapayload">SignedDataPayload</a> |
    <a href="#indexationpayload">IndexationPayload</a>
    </td>
    <td>Payload containing data. As multiple payloads are not yet supported, only <a href="#unsigneddatapayload">unsigned data payload</a> should be used.</td>
  </tr>
</table>

#### Ed25519Signature

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to value <code>1</code> to denote an Ed25519 signature.</td>
  </tr>
  <tr>
    <td>public_key</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Public key of the Ed25519 keypair which is used to verify the signature.</td>
  </tr>
  <tr>
    <td>signature</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Signature signing the serialized unsigned transaction.</td>
  </tr>
</table>

#### SignatureUnblockBlock

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to value <code>0</code> to denote a signature unlock block.</td>
  </tr>
  <tr>
    <td>signature</td>
    <td>&#10004;</td>
    <td>
        <a href="#ed25519signature">Ed25519Signature</a>
    </td>
    <td>An unlock block containing signature(s) unlocking input(s).</td>
  </tr>
</table>

#### ReferenceUnblockBlock

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Set to value <code>1</code> to denote a reference unlock block.</td>
  </tr>
  <tr>
    <td>reference</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Index of a previous unlock block.</td>
  </tr>
</table>

#### SignedTransactionPayload

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>type</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Payload type. Defaults to `0`.</td>
  </tr>
  <tr>
    <td>transaction</td>
    <td>&#10004;</td>
    <td><a href="#unsignedtransaction">UnsignedTransaction</a></td>
    <td>Essence data making up a transaction by defining its inputs and outputs and an optional payload.</td>
  </tr>
  <tr>
    <td>unblock_blocks_count</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Number of inputs specifed.</td>
  </tr>
  <tr>
    <td>unblock_blocks</td>
    <td>&#10004;</td>
    <td>
        <a href="#signatureunblockblock">SignatureUnblockBlock</a> |
        <a href="#referenceunblockblock">ReferenceUnblockBlock</a>
    </td>
    <td>Holds the unlock blocks unlocking inputs within an Unsigned Transaction</td>
  </tr>
</table>

#### Message

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>version</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Message version. Defaults to `1`.</td>
  </tr>
  <tr>
    <td>parents</td>
    <td>&#10004;</td>
    <td>string[]</td>
    <td>Message ids this message references.</td>
  </tr>
  <tr>
    <td>payload_length</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Length of the payload.</td>
  </tr>
  <tr>
    <td>payload</td>
    <td>&#10004;</td>
    <td>
        <a href="#signedtransactionpayload">SignedTransactionPayload</a> |
        <a href="#unsigneddatapayload">UnsignedDataPayload</a> |
        <a href="#signeddatapayload">SignedDataPayload</a>
    </td>
    <td>Transaction amount (exposed as a custom type with additional methods).</td>
  </tr>
  <tr>
    <td>timestamp</td>
    <td>&#10004;</td>
    <td><a href="#timestamp">Timestamp</a></td>
    <td>Transaction timestamp (exposed as a custom type with additional methods).</td>
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
    <td>Determines if the transaction is confirmed.</td>
  </tr>
  <tr>
    <td>broadcasted</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>
      Determines if the transaction was broadcasted to the network. Will be true in the following scenarios:
      <ul>
        <li>If the transaction was fetched from the network;</li>
        <li>If the transaction was successfully broadcasted from the client itself.</li>
      </ul>
      Note: This property may only be required for clients with persistent storage.
    </td>
  </tr>
  <tr>
    <td>incoming</td>
    <td>&#10004;</td>
    <td>boolean</td>
    <td>Determines if the message is an incoming transaction or not.</td>
  </tr>
  <tr>
    <td>value</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Message transfer value.</td>
  </tr>
</table>

#### StorageAdapter

<table>
  <tr>
    <td><strong>Property</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>get(key: string):<a href="#account">Account</a></td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets the account object for provided account name or id.</td>
  </tr>
  <tr>
    <td>getAll(): <a href="#account">Account</a>[]</td>
    <td>&#10004;</td>
    <td>function</td>
    <td>Gets all account objects from storage.</td>
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

*Note: Using Stronghold for storage is currently under research/development.*

Multiple storage options should be considered for managing data that requires persistence. For wallet basic metadata, such as user settings or theming options, a simple key-value [storage](https://capacitor.ionicframework.com/docs/apis/storage/) could be leveraged. For transactions and address data management a relational database such as [SQLite](https://github.com/jepiqueau/capacitor-sqlite) can be used. What follows is an Entity Relationship (ERD) diagram that shows the logical representation of the data. An _account_ is the basic entity in this database design. It has a one-to-many relationship with _addresses_ i.e. an account could have multiple _addresses_ but also an _address_ can only belong to a single _account_. An _account_ has a many-to-many relationship with _transactions_ i.e. an _account_ could have multiple _transactions_ but it’s possible that a _transaction_ belongs to multiple _accounts_. To accommodate for that, an additional table is added that stores account ids against transaction ids (hashes).  

A storage adapter is required by the Rust layer because the storage operations (read/write) will be (mostly) done from that layer. A generic storage adapter is defined [here](#storageadapter).  

![Entity Relationship Diagram](erd.jpg)

## Storage Adapter

The package should have a default opinionated storage mechanism but should also provide the ability for users to override the storage by specifying an adapter. As a default option, a relational database such as [SQLite](https://www.sqlite.org/index.html) can be used.  

See <a href="#storageAdapter">storage adapter</a> for adapter interface.

## Account

### API

#### Initialisation 

Initialises account
There are several scenarios in which an account can be initialised:

*   _Seed generated outside the Stronghold_:  In this case, the account should be initialised with a seed. It should communicate with the Stronghold using its “importAccount” method and should expect an “id” as a response; 
*   _Seed generated inside the Stronghold_: In this case, the account should be initialised without a seed. It should communicate with the Stronghold using its “createAccount” method and should expect an “id” in response;
*   _Importing accounts from Stronghold backup_: In this case, the account should receive all initialisation properties from the Stronghold. Note that during backup, these configuration settings should be passed to the Stronghold. See [import_accounts()](#import_accounts).

The following should be considered when initialising an account:

*   An account should never be initialised directly. The only way an account can be initialized is through the [add_account()](#add_account) method;
*   An account should always be initialised after a successful response from the Stronghold. If the Stronghold fails to create an account, the account initialisation should error out. If the Stronghold successfully creates an account, the account should be stored in the persistent storage. Upon a successful store operation, the user should be returned an account object;
*   If a `provider` property is not passed, a random node should be selected from the `nodes` property;
*   If a `type` property is not passed, `"default”` should be used as an account type;
*   `quorum_size` and `quorum_threshold` should be validated. For example, `quorum_size` should not be greater than the number of nodes provided by the user.
*   The `nodes` property should validate and remove duplicate node URLs;
*   All the properties of the returned account object should be read-only. It should not be possible to manipulate them directly.

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
    <td><a href="#accountconfiguration">AccountConfig</a></td>
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
    <td>account</td>
    <td><a href="#accountobject">Account</a></td>
    <td colspan="3">Account instance.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### sync_addresses() 

Syncs addresses with the Tangle. The method should ensure that the wallet's local state contains all used addresses and an unused address. 
 
The following should be considered when implementing this method:

*   The updated address history should not be written down in the database/persistent storage. Instead the method should only return the updated address history (with transaction hashes).  This will ensure that there are no partial writes to the database;
*   To sync addresses for an account from scratch, index = 0 and gap_limit = 10 should be provided;
*   To sync addresses from the latest address, index = latest address index and gap_limit = 1 should be provided. 

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
    <td>index</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Address index. By default the length of addresses stored for this account should be used as an index.</td>
  </tr>
  <tr>
    <td>gap_limit</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Number of address indexes that are generated.</td>
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
    <td>addresses</td>
    <td><a href="#address">Address</a>[]</td>
    <td colspan="3">Address history upto latest unused address.</td>
  </tr>
  <tr>
    <td>ids</td>
    <td>string[]</td>
    <td colspan="3">Message ids associated with the addresses.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Private</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#get_address_balances">get_address_balances()</a></li>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages">find_messages()</a></li>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_outputs">find_outputs()</a></li>
      </ul>
    </td>
  </tr>
</table>

#### sync_messages() 

Syncs messages with the Tangle. The method should ensure that the wallet's local state has all messages associated with the address history. 

The following should be considered when implementing this method:

*   The updated message history should not be written down in the database/persistent storage. Instead the method should only return the updated message history (with message ids);
*   This method should check if there are any local messages (with “broadcasted: false”) matching the messages fetched from the network. If there are such messages, their “broadcasted” property should be set to true;
*   For newly-confirmed messages, the method should ensure that it updates the “confirmed” property of all its reattachments.

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
    <td>ids</td>
    <td>&#10004;</td>
    <td>string[]</td>
    <td>Message ids. New message ids should be calculated by running a difference of local message ids with latest message ids on the Tangle.</td>
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
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">Message history</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Private</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages">find_messages()</a></li>
      </ul>
    </td>
  </tr>
</table>

#### select_inputs() 

Selects inputs for funds transfer.

Note: This method should only be used internally by [send()](#send). Also, the input selection method should ensure that the recipient address doesn’t match the remainder address. 

See [Input Selection Process](#input-selection) for implementation details.

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
    <td><a href="#address">Address</a>[]</td>
    <td colspan="3">Selected Inputs</td>
  </tr>
  <tr>
    <td>remainder</td>
    <td><a href="#address">Address</a></td>
    <td colspan="3">Remainder address object. Empty or null if there’s no need for a remainder</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Private</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### send() 

Sends a message to the Tangle.  

Note: This method should only be exposed as a successful response from [sync()](#sync). 

Currently, it is not possible to send multiple payloads. 

The process for sending a value transaction:
*   Ensure `amount` is not set to zero;
*   Ensure `amount` does not exceed the total balance;
*   Ensure recipient address has correct checksum;
*   Validate `data` property semantics and size;
*   Select inputs by using [select_inputs()](#selectinputs);
*   Pass the seralized [unsigned transaction](#unsignedtransaction) to the Stronghold for signing with its “signTransaction” method;
*   Perform proof-of-work. `pow` property in the account object should determine if the proof of work should be offloaded;
*   Once proof-of-work is successfully performed, the message should be validated and stored in the persistent storage;
*   After persisting the transaction, it should be broadcasted to the network;
*   In the event of a broadcast error, there should be (three) attempts for automatic rebroadcasting. If all attempts fail, the send process should terminate and it should be left to the user to retry the failed message. For failed messages, the “broadcasted” property in the transaction objects should be set to false. 

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
    <td><a href="#transfer">Transfer</a></td>
    <td>Transfer object.</td>
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
    <td>message</td>
    <td><a href="#message">Message</a></td>
    <td colspan="3">Newly made message.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Private</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages">find_messages()</a></li>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#send">send()</a></li>
      </ul>
    </td>
  </tr>
</table>

#### retry() 

Rebroadcasts failed message.

Note: This method should only be exposed as a successful response from [sync()](#sync). 

The process for retrying a failed message:

*   Get message by using [get_message()](#getmessage);
*   Rebroadcast message;
*   Update account in persistent storage.

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
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Message id</td>
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
    <td>message</td>
    <td><a href="#message">Message</a></td>
    <td colspan="3">Newly made message.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Private</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#post_message">post_message()</a></li>
      </ul>
    </td>
  </tr>
</table>

#### sync()

Syncs account with the Tangle. The account syncing process should ensure that the latest metadata (balance, messages) associated with an account is fetched from the Tangle and stored locally.  
Note that it is a proposed design decision to enforce account syncing before every send. An alternative way would be to have the _send_ method always exposed and internally ensuring that the account is synced before every message. 

The process for account syncing:_

*   Sync addresses using [sync_addresses()](#syncaddresses);
*   Sync messages using [sync_messages()](#syncmessages);
*   Store updated addresses and messages information in persistent storage (if not explicitly set otherwise by the user). 

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
    <td>index</td>
    <td>&#10008;</td>
    <td>number</td>
    <td>Address index. By default the number of addresses stored for this account should be used as an index.</td>
  </tr>
  <tr>
    <td>gap_limit</td>
    <td>&#10008;</td>
    <td>number</td>
    <td>Number of address indexes that are generated.</td>
  </tr>
  <tr>
    <td>skip_persistence</td>
    <td>&#10008;</td>
    <td>boolean</td>
    <td>
      Skips write to the database. This will be useful if a user wants to scan the Tangle for further addresses to find balance. See the
      <a href="https://docs.iota.org/docs/wallets/0.1/trinity/how-to-guides/perform-a-snapshot-transition">snapshot transition</a> feature provided by Trinity.
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
    <td>account</td>
    <td><a href="#syncedaccountobject">SyncedAccount</a></td>
    <td colspan="3">Synced account object.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#find_messages">find_messages()</a></li>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#get_address_balances">get_address_balances()</a></li>
      </ul>
    </td>
  </tr>
</table>

####  reattach() 

Reattaches unconfirmed message to the Tangle. 
The following should be considered when implementing this method:

*   Only an unconfirmed message can be reattached. The method should validate the confirmation state of the provided transaction. If a confirmed message id is provided, the method should error out;
*   The method should also validate if reattachment is necessary, by checking if the message falls below max depth. The criteria for whether the message has fallen below max depth is determined through its timestamp. If 11 minutes have passed since the timestamp of the most recent (reattachment), the message can be be reattached. See [this](https://github.com/iotaledger/trinity-wallet/blob/3fab4f671c97e805a2b0ade99b4abb8b508c2842/src/shared/libs/iota/transfers.js#L141) implementation for reference;
*   Once reattached, the message should be stored in the persistent storage;
*   If the message was reattached via polling, a [reattachment](#monitor-for-reattachments) event should be emitted to notify all subscribers. 

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
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Message id.</td>
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
    <td>message</td>
    <td><a href="#message">Message</a></td>
    <td colspan="3">Newly reattached message.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
      <ul>
        <li><a href="https://github.com/iotaledger/iota.rs/blob/dev/specs/iota-rs-ENGINEERING-SPEC-0000.md#reattach">reattach()</a></li>
      </ul>
    </td>
  </tr>
</table>

#### total_balance()

Gets total account balance.

Total balance should be read directly from the local storage. To read the latest account balance from the network, [sync()](#sync) should be used first. 

<table>
  <tr>
    <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td><a href="#value">Value</a></td>
    <td>Account total balance.</td>
  </tr>
  <tr>
    <td colspan="3"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### available_balance()

Gets available account balance. Available account balance is the amount users are allowed to spend. It should subtract the pending balance from the total balance. 

For example, if a user with _50i_ total account balance has made a transaction spending _30i_, the available balance should be _20i_ (i.e. 50i - 30i).

Available balance should be read directly from the local storage. To read the latest account balance from the network, [sync()](#sync) should be used first.

<table>
  <tr>
    <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td><strong>Type</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td><a href="#value">Value</a></td>
    <td>Account available balance.</td>
  </tr>
  <tr>
    <td colspan="3"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### set_alias() 

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
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_messages() 

Gets messages. Messages should be read directly from the local storage. To ensure the local database is updated with the latest messages, [sync()](#sync) should be used first.

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
    <td>Number of (most recent) messages.</td>
  </tr>
  <tr>
    <td>from</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Subset of messages. For example: count = 10, from = 5, it should return ten messages skipping the most recent five messages.</td>
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
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">All messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_received_messages()

Gets all received messages.

Messages should be read directly from the local storage. To ensure the local database is updated with the latest messages, [sync()](#sync) should be used first. 

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
    <td>Number of (most recent) received messages.</td>
  </tr>
  <tr>
    <td>from</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Subset of received messages.</td>
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
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">All received messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_sent_messages()

Gets all sent messages.

Messages should be directly read from the local storage. To ensure the local database is updated with the latest messages, [sync()](#sync) should be used first.

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
    <td>Number of (most recent) sent messages.</td>
  </tr>
  <tr>
    <td>from</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Subset of sent messages.</td>
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
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">All sent messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_failed_messages()

Gets all failed (broadcasted = false) messages. Messages should be read directly from the local storage.

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
    <td><strong>Name</strong></td>
    <td><strong>Required</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>count</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Number of (most recent) failed messages.</td>
  </tr>
  <tr>
    <td>from</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Subset of failed messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">All failed messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_unconfirmed_messages()

Gets all unconfirmed (confirmed = false) messages. Messages should be read directly from the local storage.  

<table>
  <tr>
    <td colspan="4"><strong>Returns</strong></td>
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
    <td>Number of (most recent) unconfirmed messages.</td>
  </tr>
  <tr>
    <td>from</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Subset of unconfirmed messages.</td>
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
    <td>messages</td>
    <td><a href="#message">Message</a>[]</td>
    <td colspan="3">All unconfirmed messages.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### get_message()

Gets message for provided id.

Messages should be read directly from the local storage. To ensure the local database is updated with the latest messages, [sync()](#sync) should be used first. 

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
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Message id.</td>
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
    <td>message</td>
    <td><a href="#message">Message</a></td>
    <td colspan="3">Message object.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_addresses()

Gets all addresses.

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
    <td>addresses</td>
    <td><a href="#address">Address</a>[]</td>
    <td>All addresses.</td>
  </tr>
  <tr>
    <td colspan="3"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### list_unspent_addresses()

Gets all unspent input addresses

<table>
  <tr>
    <td colspan="4"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td>Name</td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>addresses</td>
    <td><a href="#address">Address</a>[]</td>
    <td>All unspent input addresses.</td>
  </tr>
  <tr>
    <td colspan="3"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### generate_address()

Gets the latest unused address.

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
    <td>address</td>
    <td><a href="#address">Address</a></td>
    <td>A new address object.</td>
  </tr>
  <tr>
    <td colspan="3"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

## Account Manager

An account manager class should be publicly available for users. With the account manager, the user can create, update, delete or manage multiple accounts. The implementation details of a specific account should be abstracted away using this account manager wrapper. 

### API

#### Initialisation 

Initialises the account manager. Account manager initialisation should validate the adapter object semantics and should return an instance of the account manager.

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
    <td>adapter</td>
    <td>&#10008;</td>
    <td><a href="#storageadapter">Adapter</a></td>
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
    <td>manager</td>
    <td><a href="#accountsmanagerobject">AccountManager</a></td>
    <td colspan="3">Account manager instance.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### add_account()

Adds new account

See account [initialisation](#initialisation) for detailed implementation guidelines.

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
      <a href="#accountconfiguration">AccountConfig</a>
    </td>
    <td>Account configuration object.</td>
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
      <a href="#account">Account</a>
    </td>
    <td colspan="3">Newly created account.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### remove_account()

Removes an account.

The following should be considered when removing an account:
*   An account should first be removed from the Stronghold using its “removeAccount” method;
*   Once the account references have been removed from the Stronghold, the account should be deleted from the persistent storage.

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
    <td>
      { address: &lt;string> } | { alias: &lt;string> } |
      <p>
        { id: &lt;number> } |
      </p>
      <p>
        { index: &lt;number }
      </p>
    </td>
    <td>Identifier. Could be one of address, alias, id or index.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### sync_accounts() 

Syncs all stored accounts with the Tangle. Syncing should get the latest balance for all accounts and should find any new messages associated with the stored account.

See [Accounts Syncing Process](#accounts-syncing-process).

<table>
  <tr>
    <td colspan="3"><strong>Returns</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td><strong>Type</strong></td>
    <td><strong>Description</strong></td>
  </tr>
  <tr>
    <td>account</td>
    <td><a href="#syncedaccountobject">SyncedAccount</a>[]</td>
    <td>Synced accounts.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">
    <ul>
      <li><a href="sync">sync()</a></li>
    </ul>
    </td>
  </tr>
</table>

#### move()

Moves funds from one account to another. This method should leverage the [send()](#send) method from the sender account and should initiate a message to the receiver account.

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
    <td>from</td>
    <td>&#10004;</td>
    <td>
      { address: &lt;string> } | \ { alias: &lt;string> } |
      <p>
        { id: &lt;number> } |
      </p>
      <p>
        { index: &lt;number }
      </p>
    </td>
    <td>Identifier. Could be one of address, alias, id or index.</td>
  </tr>
  <tr>
    <td>to</td>
    <td>&#10004;</td>
    <td>
      { address: &lt;string> } | \ { alias: &lt;string> } |
      <p>
        { id: &lt;number> } |
      </p>
      <p>
        { index: &lt;number }
      </p>
    </td>
    <td>Identifier. Could be one of address, alias, id or index.</td>
  </tr>
  <tr>
    <td>amount</td>
    <td>&#10004;</td>
    <td>number</td>
    <td>Transaction amount.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### backup()

Safely creates a backup of the accounts to a destination. The file could simply be JSON containing the address & transaction histories for accounts.

This method should provide the Stronghold instance with the metadata of all accounts. 

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
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

#### import_accounts

Import (backed up) accounts.

**Implementation details are not finalised.**

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
<td><a href="#account">Account</a>[]
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
<td colspan="3">List of error messages [TBD]</td>
</tr>
<tr>
<td>Required client library methods</td>
<td colspan="3">None</td>
</tr>
</table>

#### get_account() 

Returns the account associated with the provided identifier.

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
   <td>account</td>
   <td>
<a href="#account">Account</a>
   </td>
   <td colspan="3">Account associated with identifier.</td>
  </tr>
  <tr>
<td colspan="4"><strong>Additional Information</strong></td>
</tr>
<tr>
<td><strong>Name</strong></td>
<td colspan="3"><strong>Description</strong></td>
</tr>
<tr>
<td>Access modifiers</td>
<td colspan="3">Public</td>
</tr>
<tr>
<td>Errors</td>
<td colspan="3">List of error messages [TBD]</td>
</tr>
<tr>
<td>Required client library methods</td>
<td colspan="3">None</td>
</tr>
</table>

#### reattach()

Reattaches an unconfirmed message.

See [reattach()](#reattach) method for implementation details. This method is a wrapper method provided for convenience. A user could directly access the [reattach()](#reattach) method on an account object. 

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
    <td>
      { address: &lt;string> } | \ { alias: &lt;string> } |
      <p>
        { id: &lt;number> } |
      </p>
      <p>
        { index: &lt;number }
      </p>
    </td>
    <td>Identifier. Could be one of address, alias, id or index.</td>
  </tr>
  <tr>
    <td>id</td>
    <td>&#10004;</td>
    <td>string</td>
    <td>Message id.</td>
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
    <td>message</td>
    <td><a href="#message">Message</a></td>
    <td colspan="3">Newly reattached message.</td>
  </tr>
  <tr>
    <td colspan="4"><strong>Additional Information</strong></td>
  </tr>
  <tr>
    <td><strong>Name</strong></td>
    <td colspan="3"><strong>Description</strong></td>
  </tr>
  <tr>
    <td>Access modifiers</td>
    <td colspan="3">Public</td>
  </tr>
  <tr>
    <td>Errors</td>
    <td colspan="3">List of error messages [TBD]</td>
  </tr>
  <tr>
    <td>Required client library methods</td>
    <td colspan="3">None</td>
  </tr>
</table>

## Events 

Events are categorised as the following:

1. Reactive messages emitted from the node software whenever the state on the node changes. For example, emitting new messages received by the node. Clients (Wallet) can subscribe to these events to get notified if any relevant change occurs on the node. See [example](https://github.com/iotaledger/wallet-spec/tree/events).
   
2. Messages emitted from the wallet library whenever there are any important state changes. Note that in cases where a user triggered action leads to a state change, the messages would not be emitted. For example, if a user explicitly triggers a [sync()](#sync) action leading to a state change, an explicit event is not necessary.

### Category 1 events

On every update sent from the node software via an event, the wallet library should update internal (persistent) storage and should also emit events via **category 2**. 

#### Monitor address for balance changes

<table>
  <tr>
   <td colspan="3"><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3" >&lt;Address : Balance></td>
   <td>
    <ul>
      <li>Index 1: Address</li>
      <li>Index 2: New balance on the address</li>
    </ul>
   </td>
  </tr>
</table>

#### Monitor address for new messages 

<table>
  <tr>
   <td colspan="3"><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">&lt;Address : Message></td>
   <td>
   <ul>
      <li>Index 1: Address</li>
      <li>Index 2: Id of the new message</li>
    </ul>
   </td>
  </tr>
</table>

#### Monitor message for confirmation state 

<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong>
   </td>
  </tr>
  <tr>
   <td colspan="3">&lt;MessageId>
   </td>
   <td>
    <ul>
      <li>Index 1: Message Id</li>
      <li>Index 2: Confirmation state</li>
    </ul>
   </td>
  </tr>
</table>

### Category 2 events

They could be triggered via events from **category 1** or through [polling](#polling). 

#### Monitor for balance changes

<table>
  <tr>
   <td colspan="3"><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">balances
   </td>
   <td>[{ accountId, address, balance }]</td>
  </tr>
</table>

#### Monitor for new messages 

<table>
  <tr>
   <td colspan="3"><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">messages</td>
   <td>[{ accountId, messages }]</td>
  </tr>
</table>

#### Monitor for confirmation state 

<table>
  <tr>
   <td colspan="3"><strong>Event</strong></td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">confirmations</td>
   <td>[{ accountId, messages  }]</td>
  </tr>
</table>

#### Monitor for reattachments 

<table>
  <tr>
   <td colspan="3"><strong>Event</strong></td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">reattachments</td>
   <td>[{ accountId, messages  }]</td>
  </tr>
</table>

#### Monitor for broadcasts 

<table>
  <tr>
   <td colspan="3" ><strong>Event</strong>
   </td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">broadcasts</td>
   <td>[{ accountId, messages  }]</td>
  </tr>
</table>

#### Monitor for errors 

<table>
  <tr>
   <td colspan="3" ><strong>Event</strong></td>
   <td><strong>Returned Data</strong></td>
  </tr>
  <tr>
   <td colspan="3">error</td>
   <td>{ type, error  }</td>
  </tr>
</table>

## Privacy

To maintain the financial privacy of wallet users, the application/wallet should enforce strategies that will guarantee a certain level of anonymity. These strategies should be followed:

<ul>
  <li>The wallet should only use a single address per message i.e. if an address is already used in a message, it should not be used as a deposit address and instead a new address should be generated;</li>
  <li>The input selection strategy should expose as little information as possible. See input selection for details.</li>
</ul>

Some other privacy enhancing techniques can be found [here](https://docs.google.com/document/d/1frk4r1Eq4hnGGOiKWkDiGTK5QQxKbfrvl7Iol7OZ-dc/edit#). 

## Input Selection

The goal of input selection is to avoid remainder addresses. The remainder output leaves a clue to the user's future spends. There should be a standardised input selection strategy used by the wallet. The steps for input selection are as follows:

1. Try to select an input with an exact match. For example, if a user intends to spend _X_ iotas, the wallet should try to find an address that has _X_ iotas as available balance;
2. If the previous step fails, try to select a combination of inputs that satisfy the amount leaving no change. For example, consider a scenario where the wallet with account name _Foo_ has three addresses _A_, _B_ and _C_ with _10_, _20_ and _50_ IOTA respectively. If a user intends to spend _X = 30_ IOTA, the application should search for an exact match (step no. 1). Clearly, in this case, no address balance matches _X_, therefore, the wallet should search for a subset of addresses with an accumulated balance of _X_. In this scenario, _A_ and _B_;
3. If both the previous steps fail, the wallet should select a combination of inputs that produce the minimum remainder. 

A reference implementation of different input selection algorithms for Bitcoin can be found [here](https://github.com/bitcoinjs/coinselect).

Also, the implementation of step no. 2 is quite similar to the [subset sum problem](https://en.wikipedia.org/wiki/Subset_sum_problem). Given a _total_ and a set of non-negative numbers (_inputs_), we need to determine if there is a subset which adds up to the _total_.

## Account Syncing Process

The account syncing process should detect all (used) accounts on a seed with their corresponding address and message history. Once, all accounts and histories are detected, the wallet should accumulate total balance. The syncing process should work as follows: 

1. Start with the account at index 0, generate [gap limit](https://blog.blockonomics.co/bitcoin-what-is-this-gap-limit-4f098e52d7e1) number of addresses, default to 20;
2. Check for messages and balances on the generated addresses;
3. If there are no messages and balances of 0 on all addresses, the process for generating addresses and finding messages and balances should be stopped; 
4. If any address has balance or associated messages, generate gap limit number of addresses from the index of the last address with messages or balance; 
5. Steps (1-4) should also be peformed for account at index 1. The general idea is that _n + 1_ accounts should be checked if account _n_ has any messages or balance.

Treat accounts like addresses. Only allow 1 latest unused account.

_Scenario 1_: Wallet message and address history stored in Stronghold backup

*   Start syncing from the latest address index stored in the Stronghold backup
*   Also provide a “Full sync” function to resync from index 0 across all accounts
*   Also provide “Find more history” function to sync a further 50 addresses

_Scenario 2_: User has no backup file

*   Start syncing from account 0 address 0

## Polling

 A background process that automatically performs several tasks periodically should be part of the wallet library. The goal of the background process is to perform the following tasks:  

*   _Sync accounts_: The background process should sync all accounts with the network. This should be done using the [sync_accounts()](#syncaccounts) method. If new messages are detected, a [messages](#monitor-for-new-messages) event should be used to notify all subscribers. If new balances are detected, a [balances](#monitor-for-balance-changes) event should be used to notify all subscribers. If new confirmations are detected, a [confirmations](#monitor-for-confirmation-state) event should be used to notify all subscribers; 

Note that if there are multiple failed messages, priority should be given to the old ones. 
*   _Reattach_: The background process should check if there are any (unconfirmed) messages that require reattachments. The detailed implementation flow for reattachment can be found [here](#reattach). 

The following should be considered for implementation:

*   Invoking a task explicitly that is already being performed through polling should lead to an error. For example, if the polling process is already syncing accounts and a user explicitly calls [sync()](#sync), it should throw an error;
*   Errors during the polling process should be communicated to subscribers via error events.

The background process should have a recurring checker that sequentially performs all the above tasks. The implementation should ensure that future tasks can easily be added to the background process. For reference, see Trinity’s [implementation](https://github.com/iotaledger/trinity-wallet/blob/develop/src/mobile/src/ui/components/Poll.js) of the poll component. 