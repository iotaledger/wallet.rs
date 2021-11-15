# Wallet Library Spec

## Table of Contents <!-- omit in toc -->
  - [Introduction](#introduction)
  - [Considerations](#considerations)
  - [Naming Conventions](#naming-conventions)
   - [account](#account)
      - [AccountInitialiser](#AccountInitialiser)
      - [Account](#Account)
      - [AccountHandle](#AccountHandle)
      - [AccountBalance](#AccountBalance)
   - [account_manager](#account_manager)
      - [AccountManagerBuilder](#AccountManagerBuilder)
      - [MigrationBundle](#MigrationBundle)
      - [MigratedBundle](#MigratedBundle)
      - [MinedBundle](#MinedBundle)
      - [AccountManager](#AccountManager)
      - [AccountsSynchronizer](#AccountsSynchronizer)
   - [client](#client)
      - [AddressOutput](#AddressOutput)
      - [AddressBuilder](#AddressBuilder)
      - [AddressWrapper](#AddressWrapper)
      - [Address](#Address)
      - [client.rs](#client.rs)
      - [ClientOptionsBuilder](#ClientOptionsBuilder)
      - [BrokerOptions](#BrokerOptions)
      - [NodeAuth](#NodeAuth)
      - [Node](#Node)
      - [ClientOptions](#ClientOptions)
      - [TransferOutput](#TransferOutput)
      - [TransferBuilder](#TransferBuilder)
      - [TransferBuilderWrapper](#TransferBuilderWrapper)
      - [Transfer](#Transfer)
      - [Value](#Value)
      - [TransactionSignatureLockedSingleOutput](#TransactionSignatureLockedSingleOutput)
      - [TransactionSignatureLockedDustAllowanceOutput](#TransactionSignatureLockedDustAllowanceOutput)
      - [TransactionUtxoInput](#TransactionUtxoInput)
      - [TransactionRegularEssence](#TransactionRegularEssence)
      - [MessageTransactionPayload](#MessageTransactionPayload)
      - [MessageMilestonePayloadEssence](#MessageMilestonePayloadEssence)
      - [MessageMilestonePayload](#MessageMilestonePayload)
      - [MessageMigratedFundsEntry](#MessageMigratedFundsEntry)
      - [MessageReceiptPayload](#MessageReceiptPayload)
      - [Message](#Message)
      - [AccountToCreate](#AccountToCreate)
      - [AccountDto](#AccountDto)
      - [Response](#Response)
      - [MigrationInputDto](#MigrationInputDto)
      - [MigrationDataDto](#MigrationDataDto)
   - [event](#event)
      - [event.rs](#event.rs)
      - [BalanceChange](#BalanceChange)
      - [BalanceEvent](#BalanceEvent)
      - [AddressConsolidationNeeded](#AddressConsolidationNeeded)
      - [LedgerAddressGeneration](#LedgerAddressGeneration)
      - [TransactionEvent](#TransactionEvent)
      - [TransactionConfirmationChangeEvent](#TransactionConfirmationChangeEvent)
      - [TransactionReattachmentEvent](#TransactionReattachmentEvent)
      - [AddressData](#AddressData)
      - [PreparedTransactionData](#PreparedTransactionData)
      - [TransactionIO](#TransactionIO)
      - [TransferProgress](#TransferProgress)
      - [MigrationProgress](#MigrationProgress)
      - [monitor.rs](#monitor.rs)
   - [migration](#migration)
      - [MigrationAddress](#MigrationAddress)
      - [MigrationData](#MigrationData)
      - [MigrationBundle](#MigrationBundle)
   - [signing](#signing)
      - [TransactionInput](#TransactionInput)
      - [GenerateAddressMetadata](#GenerateAddressMetadata)
   - [storage](#storage)
      - [MessageIndexation](#MessageIndexation)
   - [sync](#sync)
      - [AddressInputs](#AddressInputs)
      - [Input](#Input)
      - [Remainder](#Remainder)
      - [AccountSynchronizer](#AccountSynchronizer)
      - [SyncedAccount](#SyncedAccount)

## Introduction

The wallet library is a stateful package with a standardised interface to build applications with IOTA value transactions. The package will be compatible with different platforms such as web, desktop and mobile. 

The package introduces the concept of an _account_. An account is a reference or a label to a [seed](https://chrysalis.docs.iota.org/guides/dev_guide#seed). An account has certain properties such as [addresses](https://github.com/Wollac/protocol-rfcs/blob/bech32-address-format/text/0020-bech32-address-format/0020-bech32-address-format.md) and [messages](https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md). An account has various possible behaviours, including moving funds, looking for new messages, and making copies of message histories. An account should also be able to provide a degree of financial privacy and this should not incur any overhead. 

A similar [package](https://docs.iota.org/docs/client-libraries/0.1/account-module/introduction/overview) was previously developed but this becomes obsolete with the introduction of Ed25519 signatures. The previous account package was limited to a single account. As an improvement, the (new) package will be able to manage multiple accounts. 

To summarize, the main motivation behind this package is to offer a simplified (stateful) approach to handle IOTA payments.

## Considerations

*   Seeds should be stored and managed separately in a secure enclave and should never leave the secure environment. Secure enclaves include software enclaves such as IOTA’s Rust-based Stronghold library or hardware enclaves such as a Ledger Nano;
*   The secure enclave should have the ability to generate addresses and sign messages upon receipt of a message, and return the output in a message. If the secure enclave is initialised with a pre-generated seed, the sender process should immediately remove the seed traces from memory. 

## Naming Conventions

The primary language is [Rust](https://github.com/rust-lang/rust). Therefore, standard Rust naming [conventions](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html) are followed. All interfaces (types) use _CamelCase_ while all function and variable names use _snake\_case_.

## Account

#### AccountInitialiser

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#signer_type()">signer_type()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the account type.</td>
      </tr>
  <tr>
        <td><a href="#alias()">alias()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Defines the account alias. If not defined, we'll generate one.</td>
      </tr>
  <tr>
        <td><a href="#created_at()">created_at()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Time of account creation.</td>
      </tr>
  <tr>
        <td><a href="#messages()">messages()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Messages associated with the seed.</td>
      </tr>
  <tr>
        <td><a href="#addresses()">addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The account can be initialised with locally stored address history.</td>
      </tr>
  <tr>
        <td><a href="#skip_persistence()">skip_persistence()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skips storing the account to the database.</td>
      </tr>
  <tr>
        <td><a href="#allow_create_multiple_empty_accounts()">allow_create_multiple_empty_accounts()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Enables creating multiple accounts without history.</td>
      </tr>
  <tr>
        <td><a href="#initialise()">initialise()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises the account.</td>
      </tr>
</table>


#### Account

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#bech32_hrp()">bech32_hrp()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Returns the address bech32 human readable part.</td>
      </tr>
  <tr>
        <td><a href="#latest_address()">latest_address()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Returns the most recent address of the account.</td>
      </tr>
  <tr>
        <td><a href="#balance()">balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the account balance information.</td>
      </tr>
  <tr>
        <td><a href="#set_alias()">set_alias()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Updates the account alias.</td>
      </tr>
  <tr>
        <td><a href="#set_client_options()">set_client_options()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Updates the account's client options.</td>
      </tr>
  <tr>
          <td><a href="#list_messages()">list_messages()</a></td>
          <td>&#10004;</td>
          <td>function</td>
          <td>Gets a list of transactions on this account.</td>
        </tr>
  <tr>
        <td><a href="#list_spent_addresses()">list_spent_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the spent addresses.</td>
      </tr>
  <tr>
        <td><a href="#list_unspent_addresses()">list_unspent_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the spent addresses.</td>
      </tr>
  <tr>
        <td><a href="#get_message()">get_message()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets a message with the given id associated with this account.</td>
      </tr>
  <tr>
        <td><a href="#get_node_info()">get_node_info()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the node info from /api/v1/info endpoint.</td>
      </tr>
  <tr>
        <td><a href="#address_available_balance()">address_available_balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the available balance on the given address.</td>
      </tr>
</table>


#### AccountHandle

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#sync()">sync()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Returns the builder to setup the process to synchronize this account with the Tangle.</td>
      </tr>
  <tr>
        <td><a href="#consolidate_outputs()">consolidate_outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Consolidate account outputs.</td>
      </tr>
  <tr>
        <td><a href="#transfer()">transfer()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Send messages.</td>
      </tr>
  <tr>
        <td><a href="#retry()">retry()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Retry message.</td>
      </tr>
  <tr>
        <td><a href="#promote()">promote()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Promote message.</td>
      </tr>
  <tr>
        <td><a href="#reattach()">reattach()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Reattach message.</td>
      </tr>
  <tr>
        <td><a href="#generate_address()">generate_address()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets a new unused address and links it to this account.</td>
      </tr>
  <tr>
        <td><a href="#generate_addresses()">generate_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets amount new unused addresses and links them to this account.</td>
      </tr>
  <tr>
        <td><a href="#get_unused_address()">get_unused_address()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Synchronizes the account addresses with the Tangle and returns the latest address in the account,</td>
      </tr>
  <tr>
        <td><a href="#is_latest_address_unused()">is_latest_address_unused()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Syncs the latest address with the Tangle and determines whether it's unused or not.</td>
      </tr>
  <tr>
        <td><a href="#latest_address()">latest_address()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#latest_address](struct.Account.html#method.latest_address).</td>
      </tr>
  <tr>
        <td><a href="#balance()">balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#balance](struct.Account.html#method.balance).</td>
      </tr>
  <tr>
        <td><a href="#set_alias()">set_alias()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#set_alias](struct.Account.html#method.set_alias).</td>
      </tr>
  <tr>
        <td><a href="#set_client_options()">set_client_options()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#set_client_options](struct.Account.html#method.set_client_options).</td>
      </tr>
  <tr>
          <td><a href="#list_messages()">list_messages()</a></td>
          <td>&#10004;</td>
          <td>function</td>
          <td>Bridge to [Account#list_messages](struct.Account.html#method.list_messages).</td>
        </tr>
  <tr>
        <td><a href="#list_spent_addresses()">list_spent_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).</td>
      </tr>
  <tr>
        <td><a href="#list_unspent_addresses()">list_unspent_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).</td>
      </tr>
  <tr>
        <td><a href="#get_message()">get_message()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Bridge to [Account#get_message](struct.Account.html#method.get_message).</td>
      </tr>
  <tr>
          <td><a href="#get_node_info()">get_node_info()</a></td>
          <td>&#10004;</td>
          <td>function</td>
          <td>Bridge to [Account#get_node_info](struct.Account.html#method.get_node_info).</td>
        </tr>
</table>


#### AccountBalance

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>total</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Account's total balance.</td>
      </tr>
  <tr>
        <td>available</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>The available balance is the balance users are allowed to spend.</td>
      </tr>
  <tr>
        <td>incoming</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Balances from message with `incoming: true`.</td>
      </tr>
  <tr>
        <td>outgoing</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Balances from message with `incoming: false`.</td>
      </tr>
</table>

### AccountInitialiser

#### signer_type()

Sets the account type.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>signer_type</td>
        <td><a href="#signertype">SignerType</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### alias()

Defines the account alias. If not defined, we'll generate one.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>alias</td>
        <td>impl</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### created_at()

Time of account creation.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>created_at</td>
        <td><a href="#datetime<local>">DateTime&lt;Local&gt;</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### messages()

Messages associated with the seed.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>messages</td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### addresses()

The account can be initialised with locally stored address history.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>addresses</td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### skip_persistence()

Skips storing the account to the database.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### allow_create_multiple_empty_accounts()

Enables creating multiple accounts without history.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### initialise()

Initialises the account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accounthandle">AccountHandle</a></td>
        <td></td>
      </tr>
</table>

### Account

#### bech32_hrp()

Returns the address bech32 human readable part.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

#### latest_address()

Returns the most recent address of the account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### balance()

Gets the account balance information.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountbalance">AccountBalance</a></td>
        <td></td>
      </tr>
</table>

#### set_alias()

Updates the account alias.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>alias</td>
        <td>impl</td>
        <td></td>
      </tr>
</table>

#### set_client_options()

Updates the account's client options.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>options</td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td></td>
      </tr>
</table>

#### list_messages()

Gets a list of transactions on this account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>count</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>from</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_type</td>
        <td><a href="#messagetype">MessageType</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### list_spent_addresses()

Gets the spent addresses.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### list_unspent_addresses()

Gets the spent addresses.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### get_message()

Gets a message with the given id associated with this account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_id</td>
        <td><a href="#messageid">MessageId</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### get_node_info()

Gets the node info from /api/v1/info endpoint.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>url</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>jwt</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>auth</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#nodeinfowrapper">NodeInfoWrapper</a></td>
        <td></td>
      </tr>
</table>

#### address_available_balance()

Gets the available balance on the given address.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>u64</td>
        <td></td>
      </tr>
</table>

### AccountHandle

#### sync()

Returns the builder to setup the process to synchronize this account with the Tangle.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountsynchronizer">AccountSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### consolidate_outputs()

Consolidate account outputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>include_dust_allowance_outputs</td>
        <td>bool</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### transfer()

Send messages.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>transfer_obj</td>
        <td><a href="#transfer">Transfer</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### retry()

Retry message.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_id</td>
        <td><a href="#messageid">MessageId</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### promote()

Promote message.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_id</td>
        <td><a href="#messageid">MessageId</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### reattach()

Reattach message.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_id</td>
        <td><a href="#messageid">MessageId</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### generate_address()

Gets a new unused address and links it to this account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### generate_addresses()

Gets amount new unused addresses and links them to this account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>amount</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### get_unused_address()

Synchronizes the account addresses with the Tangle and returns the latest address in the account,
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### is_latest_address_unused()

Syncs the latest address with the Tangle and determines whether it's unused or not.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>bool</td>
        <td></td>
      </tr>
</table>

#### latest_address()

Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### balance()

Bridge to [Account#balance](struct.Account.html#method.balance).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountbalance">AccountBalance</a></td>
        <td></td>
      </tr>
</table>

#### set_alias()

Bridge to [Account#set_alias](struct.Account.html#method.set_alias).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>alias</td>
        <td>impl</td>
        <td></td>
      </tr>
</table>

#### set_client_options()

Bridge to [Account#set_client_options](struct.Account.html#method.set_client_options).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>options</td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td></td>
      </tr>
</table>

#### list_messages()

Bridge to [Account#list_messages](struct.Account.html#method.list_messages).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>count</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>from</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_type</td>
        <td><a href="#messagetype">MessageType</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### list_spent_addresses()

Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### list_unspent_addresses()

Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

#### get_message()

Bridge to [Account#get_message](struct.Account.html#method.get_message).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>message_id</td>
        <td><a href="#messageid">MessageId</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#message">Message</a></td>
        <td></td>
      </tr>
</table>

#### get_node_info()

Bridge to [Account#get_node_info](struct.Account.html#method.get_node_info).
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>url</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>jwt</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>auth</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#nodeinfowrapper">NodeInfoWrapper</a></td>
        <td></td>
      </tr>
</table>


## Account Manager

An account manager class should be publicly available for users. With the account manager, the user can create, update, delete or manage multiple accounts. The implementation details of a specific account should be abstracted using this account manager wrapper.

#### AccountManagerBuilder

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises a new instance of the account manager builder with the default storage adapter.</td>
      </tr>
  <tr>
        <td><a href="#with_storage()">with_storage()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the storage config to be used.</td>
      </tr>
  <tr>
        <td><a href="#with_polling_interval()">with_polling_interval()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the polling interval.</td>
      </tr>
  <tr>
        <td><a href="#with_skip_polling()">with_skip_polling()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skip polling</td>
      </tr>
  <tr>
        <td><a href="#with_output_consolidation_threshold()">with_output_consolidation_threshold()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the number of outputs an address must have to trigger the automatic consolidation process.</td>
      </tr>
  <tr>
        <td><a href="#with_automatic_output_consolidation_disabled()">with_automatic_output_consolidation_disabled()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Disables the automatic output consolidation process.</td>
      </tr>
  <tr>
        <td><a href="#with_sync_spent_outputs()">with_sync_spent_outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Enables fetching spent output history on sync.</td>
      </tr>
  <tr>
        <td><a href="#with_event_persistence()">with_event_persistence()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Enables event persistence.</td>
      </tr>
  <tr>
        <td><a href="#with_multiple_empty_accounts()">with_multiple_empty_accounts()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Enables creating multiple accounts without history.</td>
      </tr>
  <tr>
        <td><a href="#finish()">finish()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Builds the manager.</td>
      </tr>
</table>


#### AccountManager

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#builder()">builder()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises the account manager builder.</td>
      </tr>
  <tr>
        <td><a href="#get_migration_data()">get_migration_data()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the legacy migration data for the seed.</td>
      </tr>
  <tr>
        <td><a href="#send_migration_bundle()">send_migration_bundle()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sends the migration bundle to the given node.</td>
      </tr>
  <tr>
        <td><a href="#delete()">delete()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Deletes the storage.</td>
      </tr>
  <tr>
        <td><a href="#stop_background_sync()">stop_background_sync()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Stops the background polling and MQTT monitoring.</td>
      </tr>
  <tr>
        <td><a href="#is_latest_address_unused()">is_latest_address_unused()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Determines whether all accounts has the latest address unused.</td>
      </tr>
  <tr>
        <td><a href="#set_client_options()">set_client_options()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the client options for all accounts.</td>
      </tr>
  <tr>
        <td><a href="#store_mnemonic()">store_mnemonic()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Stores a mnemonic for the given signer type.</td>
      </tr>
  <tr>
        <td><a href="#generate_mnemonic()">generate_mnemonic()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Generates a new mnemonic.</td>
      </tr>
  <tr>
        <td><a href="#create_account()">create_account()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Adds a new account.</td>
      </tr>
  <tr>
        <td><a href="#sync_accounts()">sync_accounts()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Syncs all accounts.</td>
      </tr>
  <tr>
        <td><a href="#get_accounts()">get_accounts()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets all accounts from storage.</td>
      </tr>
  <tr>
        <td><a href="#get_seed_checksum()">get_seed_checksum()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Get seed checksum</td>
      </tr>
</table>


#### AccountsSynchronizer

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#gap_limit()">gap_limit()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Number of address indexes that are generated.</td>
      </tr>
  <tr>
        <td><a href="#address_index()">address_index()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initial address index to start syncing.</td>
      </tr>
  <tr>
        <td><a href="#skip_account_discovery()">skip_account_discovery()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skips the account discovery process.</td>
      </tr>
  <tr>
        <td><a href="#skip_change_addresses()">skip_change_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skip syncing existing change addresses.</td>
      </tr>
  <tr>
        <td><a href="#account_discovery_threshold()">account_discovery_threshold()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the minimum number of accounts to check on the discovery process.</td>
      </tr>
  <tr>
        <td><a href="#execute()">execute()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Syncs the accounts with the Tangle.</td>
      </tr>
</table>

### AccountManagerBuilder

#### new()

Initialises a new instance of the account manager builder with the default storage adapter.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_storage()

Sets the storage config to be used.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>storage_folder</td>
        <td>impl</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>password</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_polling_interval()

Sets the polling interval.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>polling_interval</td>
        <td><a href="#duration">Duration</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_skip_polling()

Skip polling
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_output_consolidation_threshold()

Sets the number of outputs an address must have to trigger the automatic consolidation process.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>threshold</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_automatic_output_consolidation_disabled()

Disables the automatic output consolidation process.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_sync_spent_outputs()

Enables fetching spent output history on sync.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_event_persistence()

Enables event persistence.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_multiple_empty_accounts()

Enables creating multiple accounts without history.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### finish()

Builds the manager.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanager">AccountManager</a></td>
        <td></td>
      </tr>
</table>

### AccountManager

#### builder()

Initialises the account manager builder.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountmanagerbuilder">AccountManagerBuilder</a></td>
        <td></td>
      </tr>
</table>

#### get_migration_data()

Gets the legacy migration data for the seed.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>finder</td>
        <td><a href="#migrationdatafinder">MigrationDataFinder</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#migrationdata">MigrationData</a></td>
        <td></td>
      </tr>
</table>

#### send_migration_bundle()

Sends the migration bundle to the given node.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>nodes</td>
        <td>False</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>hash</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>mwm</td>
        <td>u8</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#migratedbundle">MigratedBundle</a></td>
        <td></td>
      </tr>
</table>

#### delete()

Deletes the storage.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

#### stop_background_sync()

Stops the background polling and MQTT monitoring.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

#### is_latest_address_unused()

Determines whether all accounts has the latest address unused.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>bool</td>
        <td></td>
      </tr>
</table>

#### set_client_options()

Sets the client options for all accounts.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>options</td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td></td>
      </tr>
</table>

#### store_mnemonic()

Stores a mnemonic for the given signer type.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>signer_type</td>
        <td><a href="#signertype">SignerType</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>mnemonic</td>
        <td>String</td>
        <td></td>
      </tr>
</table>

#### generate_mnemonic()

Generates a new mnemonic.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

#### create_account()

Adds a new account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>client_options</td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountinitialiser">AccountInitialiser</a></td>
        <td></td>
      </tr>
</table>

#### sync_accounts()

Syncs all accounts.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### get_accounts()

Gets all accounts from storage.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accounthandle">AccountHandle</a></td>
        <td></td>
      </tr>
</table>

#### get_seed_checksum()

Get seed checksum
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>seed</td>
        <td>String</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

### AccountsSynchronizer

#### gap_limit()

Number of address indexes that are generated.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>limit</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### address_index()

Initial address index to start syncing.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address_index</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### skip_account_discovery()

Skips the account discovery process.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### skip_change_addresses()

Skip syncing existing change addresses.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### account_discovery_threshold()

Sets the minimum number of accounts to check on the discovery process.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>account_discovery_threshold</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountssynchronizer">AccountsSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### execute()

Syncs the accounts with the Tangle.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#syncedaccount">SyncedAccount</a></td>
        <td></td>
      </tr>
</table>


## Client

#### AddressOutput

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>transaction_id</td>
        <td>&#10004;</td>
        <td><a href="#transactionid">TransactionId</a></td>
        <td>Transaction ID of the output</td>
      </tr>
  <tr>
        <td>message_id</td>
        <td>&#10004;</td>
        <td><a href="#messageid">MessageId</a></td>
        <td>Message ID of the output</td>
      </tr>
  <tr>
        <td>index</td>
        <td>&#10004;</td>
        <td>u16</td>
        <td>Output index.</td>
      </tr>
  <tr>
        <td>amount</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Output amount.</td>
      </tr>
  <tr>
        <td>address</td>
        <td>&#10004;</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td>Associated address.</td>
      </tr>
  <tr>
        <td>kind</td>
        <td>&#10004;</td>
        <td><a href="#outputkind">OutputKind</a></td>
        <td>Output kind.</td>
      </tr>
  <tr>
        <td><a href="#id()">id()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The output identifier.</td>
      </tr>
</table>


#### AddressBuilder

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises a new instance of the address builder.</td>
      </tr>
  <tr>
        <td><a href="#address()">address()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Defines the address.</td>
      </tr>
  <tr>
        <td><a href="#key_index()">key_index()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the address key index.</td>
      </tr>
  <tr>
        <td><a href="#outputs()">outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the address outputs.</td>
      </tr>
  <tr>
        <td><a href="#internal()">internal()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the `internal` flag.</td>
      </tr>
  <tr>
        <td><a href="#build()">build()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Builds the address.</td>
      </tr>
</table>


#### AddressWrapper

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Create a new address wrapper.</td>
      </tr>
  <tr>
        <td><a href="#to_bech32()">to_bech32()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Encodes the address as bech32.</td>
      </tr>
</table>


#### Address

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#balance()">balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Address total balance</td>
      </tr>
</table>


#### client-client.rs

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#drop_all()">drop_all()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Drops all clients.</td>
      </tr>
</table>


#### ClientOptionsBuilder

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises a new instance of the builder.</td>
      </tr>
  <tr>
        <td><a href="#with_primary_node()">with_primary_node()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the primary node.</td>
      </tr>
  <tr>
        <td><a href="#with_primary_pow_node()">with_primary_pow_node()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the primary PoW node.</td>
      </tr>
  <tr>
        <td><a href="#with_nodes()">with_nodes()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>ClientOptions connected to a list of nodes.</td>
      </tr>
  <tr>
        <td><a href="#with_node()">with_node()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Adds a node to the node list.</td>
      </tr>
  <tr>
        <td><a href="#with_node_pool_urls()">with_node_pool_urls()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Get node list from the node_pool_urls</td>
      </tr>
  <tr>
        <td><a href="#with_node_sync_interval()">with_node_sync_interval()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Set the node sync interval</td>
      </tr>
  <tr>
        <td><a href="#with_node_sync_disabled()">with_node_sync_disabled()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Disables the node syncing process.</td>
      </tr>
  <tr>
        <td><a href="#with_mqtt_mqtt_broker_options()">with_mqtt_mqtt_broker_options()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the MQTT broker options.</td>
      </tr>
  <tr>
        <td><a href="#with_mqtt_disabled()">with_mqtt_disabled()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the MQTT broker options.</td>
      </tr>
  <tr>
        <td><a href="#with_local_pow()">with_local_pow()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets whether the PoW should be done locally or remotely.</td>
      </tr>
  <tr>
        <td><a href="#with_request_timeout()">with_request_timeout()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the request timeout.</td>
      </tr>
  <tr>
        <td><a href="#with_api_timeout()">with_api_timeout()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the request timeout for a specific API usage.</td>
      </tr>
  <tr>
        <td><a href="#build()">build()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Builds the options.</td>
      </tr>
</table>


#### BrokerOptions

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>automatic_disconnect</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not.</td>
      </tr>
  <tr>
        <td>timeout</td>
        <td>&#10004;</td>
        <td><a href="#duration">Duration</a></td>
        <td>timeout of the mqtt broker.</td>
      </tr>
  <tr>
        <td>use_ws</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Defines if websockets should be used (true) or TCP (false)</td>
      </tr>
  <tr>
        <td>port</td>
        <td>&#10004;</td>
        <td>u16</td>
        <td>Defines the port to be used for the MQTT connection</td>
      </tr>
  <tr>
        <td>max_reconnection_attempts</td>
        <td>&#10004;</td>
        <td>usize</td>
        <td>Defines the maximum reconnection attempts before it returns an error</td>
      </tr>
</table>


#### NodeAuth

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>jwt</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>JWT.</td>
      </tr>
  <tr>
        <td>basic_auth_name_pwd</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>Username and password.</td>
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
        <td><a href="#url">Url</a></td>
        <td>Node url.</td>
      </tr>
  <tr>
        <td>auth</td>
        <td>&#10004;</td>
        <td><a href="#nodeauth">NodeAuth</a></td>
        <td>Node auth options.</td>
      </tr>
  <tr>
        <td>disabled</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the node is disabled or not.</td>
      </tr>
</table>


#### ClientOptions

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#builder()">builder()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets a new client options builder instance.</td>
      </tr>
</table>


#### TransferOutput

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
        <td><a href="#nonzerou64">NonZeroU64</a></td>
        <td>The output value.</td>
      </tr>
  <tr>
        <td>address</td>
        <td>&#10004;</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td>The output address.</td>
      </tr>
  <tr>
        <td>output_kind</td>
        <td>&#10004;</td>
        <td><a href="#outputkind">OutputKind</a></td>
        <td>The output type</td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Creates a new transfer output.</td>
      </tr>
</table>


#### TransferBuilder

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises a new transfer to the given address.</td>
      </tr>
  <tr>
        <td><a href="#with_outputs()">with_outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Creates a transfer with multiple outputs.</td>
      </tr>
  <tr>
        <td><a href="#with_remainder_value_strategy()">with_remainder_value_strategy()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Sets the remainder value strategy for the transfer.</td>
      </tr>
  <tr>
        <td><a href="#with_indexation()">with_indexation()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>(Optional) message indexation.</td>
      </tr>
  <tr>
        <td><a href="#with_skip_sync()">with_skip_sync()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skip account syncing before transferring.</td>
      </tr>
  <tr>
        <td><a href="#finish()">finish()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Builds the transfer.</td>
      </tr>
</table>


#### Transfer

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#builder()">builder()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises the transfer builder.</td>
      </tr>
  <tr>
        <td><a href="#builder_with_outputs()">builder_with_outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initialises the transfer builder with multiple outputs.</td>
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
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Ititialises a new Value.</td>
      </tr>
  <tr>
        <td><a href="#with_denomination()">with_denomination()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Formats the value with its unit.</td>
      </tr>
  <tr>
        <td><a href="#without_denomination()">without_denomination()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The transaction value without its unit.</td>
      </tr>
</table>


#### TransactionUtxoInput

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>input</td>
        <td>&#10004;</td>
        <td><a href="#utxoinput">UtxoInput</a></td>
        <td>UTXO input.</td>
      </tr>
  <tr>
        <td>metadata</td>
        <td>&#10004;</td>
        <td><a href="#addressoutput">AddressOutput</a></td>
        <td>Metadata.</td>
      </tr>
</table>


#### TransactionRegularEssence

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#inputs()">inputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the transaction inputs.</td>
      </tr>
  <tr>
        <td><a href="#outputs()">outputs()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the transaction outputs.</td>
      </tr>
  <tr>
        <td><a href="#payload()">payload()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Gets the transaction chained payload.</td>
      </tr>
  <tr>
        <td><a href="#internal()">internal()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Whether the transaction is between the mnemonic accounts or not.</td>
      </tr>
  <tr>
        <td><a href="#incoming()">incoming()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Whether the transaction is incoming or outgoing.</td>
      </tr>
  <tr>
        <td><a href="#value()">value()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The transactions's value.</td>
      </tr>
  <tr>
        <td><a href="#remainder_value()">remainder_value()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The transactions's remainder value sum.</td>
      </tr>
</table>


#### MessageTransactionPayload

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#essence()">essence()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The transaction essence.</td>
      </tr>
  <tr>
        <td><a href="#unlock_blocks()">unlock_blocks()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The unlock blocks.</td>
      </tr>
</table>


#### MessageMilestonePayload

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#essence()">essence()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The milestone essence.</td>
      </tr>
  <tr>
        <td><a href="#signatures()">signatures()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The milestone signatures.</td>
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
        <td><a href="#message_type()">message_type()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The message type.</td>
      </tr>
  <tr>
        <td><a href="#response_tx()">response_tx()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The response sender.</td>
      </tr>
  <tr>
        <td><a href="#id()">id()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The message identifier.</td>
      </tr>
</table>


#### AccountToCreate

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>client_options</td>
        <td>&#10004;</td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td>The node options.</td>
      </tr>
  <tr>
        <td>alias</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The account alias.</td>
      </tr>
  <tr>
        <td>created_at</td>
        <td>&#10004;</td>
        <td><a href="#datetime<local>">DateTime&lt;Local&gt;</a></td>
        <td>The account createdAt date string.</td>
      </tr>
  <tr>
        <td>skip_persistence</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether to skip saving the account to storage or not.</td>
      </tr>
  <tr>
        <td>signer_type</td>
        <td>&#10004;</td>
        <td><a href="#signertype">SignerType</a></td>
        <td>The account's signer type.</td>
      </tr>
  <tr>
        <td>allow_create_multiple_empty_accounts</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Allow to create an account with multiple empty accounts</td>
      </tr>
</table>


#### AccountDto

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>account</td>
        <td>&#10004;</td>
        <td><a href="#account">Account</a></td>
        <td>Inner account object.</td>
      </tr>
  <tr>
        <td>messages</td>
        <td>&#10004;</td>
        <td><a href="#walletmessage">WalletMessage</a></td>
        <td>Message history.</td>
      </tr>
  <tr>
        <td><a href="#new()">new()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Creates a new instance of the account DTO.</td>
      </tr>
</table>


#### Response

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#response()">response()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>The response's type.</td>
      </tr>
</table>

### AddressOutput

#### id()

The output identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#outputid">OutputId</a></td>
        <td></td>
      </tr>
</table>

### AddressBuilder

#### new()

Initialises a new instance of the address builder.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addressbuilder">AddressBuilder</a></td>
        <td></td>
      </tr>
</table>

#### address()

Defines the address.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addressbuilder">AddressBuilder</a></td>
        <td></td>
      </tr>
</table>

#### key_index()

Sets the address key index.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>key_index</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addressbuilder">AddressBuilder</a></td>
        <td></td>
      </tr>
</table>

#### outputs()

Sets the address outputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>outputs</td>
        <td><a href="#addressoutput">AddressOutput</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addressbuilder">AddressBuilder</a></td>
        <td></td>
      </tr>
</table>

#### internal()

Sets the `internal` flag.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>internal</td>
        <td>bool</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addressbuilder">AddressBuilder</a></td>
        <td></td>
      </tr>
</table>

#### build()

Builds the address.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#address">Address</a></td>
        <td></td>
      </tr>
</table>

### AddressWrapper

#### new()

Create a new address wrapper.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#iotaaddress">IotaAddress</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>bech32_hrp</td>
        <td>String</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
</table>

#### to_bech32()

Encodes the address as bech32.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

### Address

#### balance()

Address total balance
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>u64</td>
        <td></td>
      </tr>
</table>

### client-client.rs

#### drop_all()

Drops all clients.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

### ClientOptionsBuilder

#### new()

Initialises a new instance of the builder.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_primary_node()

Sets the primary node.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>node</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_primary_pow_node()

Sets the primary PoW node.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>node</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_nodes()

ClientOptions connected to a list of nodes.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>nodes</td>
        <td>False</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_node()

Adds a node to the node list.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>node</td>
        <td>str</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_node_pool_urls()

Get node list from the node_pool_urls
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>node_pool_urls</td>
        <td>False</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_node_sync_interval()

Set the node sync interval
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>node_sync_interval</td>
        <td><a href="#duration">Duration</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_node_sync_disabled()

Disables the node syncing process.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_mqtt_mqtt_broker_options()

Sets the MQTT broker options.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>options</td>
        <td><a href="#brokeroptions">BrokerOptions</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_mqtt_disabled()

Sets the MQTT broker options.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_local_pow()

Sets whether the PoW should be done locally or remotely.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>local</td>
        <td>bool</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_request_timeout()

Sets the request timeout.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>timeout</td>
        <td><a href="#duration">Duration</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_api_timeout()

Sets the request timeout for a specific API usage.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>api</td>
        <td><a href="#api">Api</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>timeout</td>
        <td><a href="#duration">Duration</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

#### build()

Builds the options.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptions">ClientOptions</a></td>
        <td></td>
      </tr>
</table>

### ClientOptions

#### builder()

Gets a new client options builder instance.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#clientoptionsbuilder">ClientOptionsBuilder</a></td>
        <td></td>
      </tr>
</table>

### TransferOutput

#### new()

Creates a new transfer output.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>amount</td>
        <td><a href="#nonzerou64">NonZeroU64</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>output_kind</td>
        <td><a href="#outputkind">OutputKind</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferoutput">TransferOutput</a></td>
        <td></td>
      </tr>
</table>

### TransferBuilder

#### new()

Initialises a new transfer to the given address.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>amount</td>
        <td><a href="#nonzerou64">NonZeroU64</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>output_kind</td>
        <td><a href="#outputkind">OutputKind</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_outputs()

Creates a transfer with multiple outputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>outputs</td>
        <td><a href="#transferoutput">TransferOutput</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_remainder_value_strategy()

Sets the remainder value strategy for the transfer.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>strategy</td>
        <td><a href="#remaindervaluestrategy">RemainderValueStrategy</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_indexation()

(Optional) message indexation.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>indexation</td>
        <td><a href="#indexationpayload">IndexationPayload</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### with_skip_sync()

Skip account syncing before transferring.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### finish()

Builds the transfer.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transfer">Transfer</a></td>
        <td></td>
      </tr>
</table>

### Transfer

#### builder()

Initialises the transfer builder.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>amount</td>
        <td><a href="#nonzerou64">NonZeroU64</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>output_kind</td>
        <td><a href="#outputkind">OutputKind</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

#### builder_with_outputs()

Initialises the transfer builder with multiple outputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>outputs</td>
        <td><a href="#transferoutput">TransferOutput</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transferbuilder">TransferBuilder</a></td>
        <td></td>
      </tr>
</table>

### Value

#### new()

Ititialises a new Value.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>value</td>
        <td>u64</td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>unit</td>
        <td><a href="#valueunit">ValueUnit</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#value">Value</a></td>
        <td></td>
      </tr>
</table>

#### with_denomination()

Formats the value with its unit.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

#### without_denomination()

The transaction value without its unit.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>u64</td>
        <td></td>
      </tr>
</table>

### TransactionRegularEssence

#### inputs()

Gets the transaction inputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

#### outputs()

Gets the transaction outputs.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

#### payload()

Gets the transaction chained payload.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#payload">Payload</a></td>
        <td></td>
      </tr>
</table>

#### internal()

Whether the transaction is between the mnemonic accounts or not.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>bool</td>
        <td></td>
      </tr>
</table>

#### incoming()

Whether the transaction is incoming or outgoing.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>bool</td>
        <td></td>
      </tr>
</table>

#### value()

The transactions's value.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>u64</td>
        <td></td>
      </tr>
</table>

#### remainder_value()

The transactions's remainder value sum.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>u64</td>
        <td></td>
      </tr>
</table>

### MessageTransactionPayload

#### essence()

The transaction essence.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#transactionessence">TransactionEssence</a></td>
        <td></td>
      </tr>
</table>

#### unlock_blocks()

The unlock blocks.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
</table>

### MessageMilestonePayload

#### essence()

The milestone essence.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#messagemilestonepayloadessence">MessageMilestonePayloadEssence</a></td>
        <td></td>
      </tr>
</table>

#### signatures()

The milestone signatures.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#box">Box</a></td>
        <td></td>
      </tr>
</table>

### Message

#### message_type()

The message type.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#messagetype">MessageType</a></td>
        <td></td>
      </tr>
</table>

#### response_tx()

The response sender.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#unboundedsender<response>">UnboundedSender&lt;Response&gt;</a></td>
        <td></td>
      </tr>
</table>

#### id()

The message identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td>String</td>
        <td></td>
      </tr>
</table>

### AccountDto

#### new()

Creates a new instance of the account DTO.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>account</td>
        <td><a href="#account">Account</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>messages</td>
        <td><a href="#walletmessage">WalletMessage</a></td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountdto">AccountDto</a></td>
        <td></td>
      </tr>
</table>

### Response

#### response()

The response's type.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#responsetype">ResponseType</a></td>
        <td></td>
      </tr>
</table>


## Event

The library is able to listen to several supported event. As soon as the event occurs, a provided callback will be triggered.

#### event-event.rs

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#remove_balance_change_listener()">remove_balance_change_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the balance change listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_new_transaction_listener()">remove_new_transaction_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the new transaction listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_confirmation_state_change_listener()">remove_confirmation_state_change_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the new confirmation state change listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_reattachment_listener()">remove_reattachment_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the reattachment listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_broadcast_listener()">remove_broadcast_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the broadcast listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_error_listener()">remove_error_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Removes the error listener associated with the given identifier.</td>
      </tr>
  <tr>
        <td><a href="#remove_transfer_progress_listener()">remove_transfer_progress_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Remove a transfer event listener.</td>
      </tr>
  <tr>
        <td><a href="#remove_migration_progress_listener()">remove_migration_progress_listener()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Remove a migration event listener.</td>
      </tr>
</table>


#### BalanceChange

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>spent</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>The change amount if it was a spent event.</td>
      </tr>
  <tr>
        <td>received</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>The change amount if it was a receive event.</td>
      </tr>
</table>


#### BalanceEvent

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>indexation_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>Event unique identifier.</td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>address</td>
        <td>&#10004;</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td>The associated address.</td>
      </tr>
  <tr>
        <td>message_id</td>
        <td>&#10004;</td>
        <td><a href="#messageid">MessageId</a></td>
        <td>with_sync_spent_outputs).</td>
      </tr>
  <tr>
        <td>balance_change</td>
        <td>&#10004;</td>
        <td><a href="#balancechange">BalanceChange</a></td>
        <td>The balance change data.</td>
      </tr>
</table>


#### AddressConsolidationNeeded

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>address</td>
        <td>&#10004;</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td>The associated address.</td>
      </tr>
</table>


#### LedgerAddressGeneration

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>event</td>
        <td>&#10004;</td>
        <td><a href="#addressdata">AddressData</a></td>
        <td>The transfer event type.</td>
      </tr>
</table>


#### TransactionEvent

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>indexation_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>Event unique identifier.</td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>message</td>
        <td>&#10004;</td>
        <td><a href="#message">Message</a></td>
        <td>The event message.</td>
      </tr>
</table>


#### TransactionConfirmationChangeEvent

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>indexation_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>Event unique identifier.</td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>message</td>
        <td>&#10004;</td>
        <td><a href="#message">Message</a></td>
        <td>The event message.</td>
      </tr>
  <tr>
        <td>confirmed</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>The confirmed state of the transaction.</td>
      </tr>
</table>


#### TransactionReattachmentEvent

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>indexation_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>Event unique identifier.</td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>message</td>
        <td>&#10004;</td>
        <td><a href="#message">Message</a></td>
        <td>The event message.</td>
      </tr>
  <tr>
        <td>reattached_message_id</td>
        <td>&#10004;</td>
        <td><a href="#messageid">MessageId</a></td>
        <td>The id of the message that was reattached.</td>
      </tr>
</table>


#### AddressData

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
        <td>String</td>
        <td>The address.</td>
      </tr>
</table>


#### PreparedTransactionData

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>inputs</td>
        <td>&#10004;</td>
        <td><a href="#transactionio">TransactionIO</a></td>
        <td>Transaction inputs.</td>
      </tr>
  <tr>
        <td>outputs</td>
        <td>&#10004;</td>
        <td><a href="#transactionio">TransactionIO</a></td>
        <td>Transaction outputs.</td>
      </tr>
  <tr>
        <td>data</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The indexation data.</td>
      </tr>
</table>


#### TransactionIO

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
        <td>String</td>
        <td>Address</td>
      </tr>
  <tr>
        <td>amount</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Amount</td>
      </tr>
  <tr>
        <td>remainder</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Remainder</td>
      </tr>
</table>


#### TransferProgress

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>account_id</td>
        <td>&#10004;</td>
        <td>String</td>
        <td>The associated account identifier.</td>
      </tr>
  <tr>
        <td>event</td>
        <td>&#10004;</td>
        <td><a href="#transferprogresstype">TransferProgressType</a></td>
        <td>The transfer event type.</td>
      </tr>
</table>


#### MigrationProgress

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>event</td>
        <td>&#10004;</td>
        <td><a href="#migrationprogresstype">MigrationProgressType</a></td>
        <td>The transfer event type.</td>
      </tr>
</table>


#### event-monitor.rs

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#unsubscribe()">unsubscribe()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Unsubscribe from all topics associated with the account.</td>
      </tr>
  <tr>
        <td><a href="#monitor_account_addresses_balance()">monitor_account_addresses_balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Monitor account addresses for balance changes.</td>
      </tr>
  <tr>
        <td><a href="#monitor_address_balance()">monitor_address_balance()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Monitor address for balance changes.</td>
      </tr>
</table>

### event-event.rs

#### remove_balance_change_listener()

Removes the balance change listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_new_transaction_listener()

Removes the new transaction listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_confirmation_state_change_listener()

Removes the new confirmation state change listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_reattachment_listener()

Removes the reattachment listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_broadcast_listener()

Removes the broadcast listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_error_listener()

Removes the error listener associated with the given identifier.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_transfer_progress_listener()

Remove a transfer event listener.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

#### remove_migration_progress_listener()

Remove a migration event listener.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>id</td>
        <td><a href="#eventid">EventId</a></td>
        <td></td>
      </tr>
</table>

### event-monitor.rs

#### unsubscribe()

Unsubscribe from all topics associated with the account.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>account_handle</td>
        <td><a href="#accounthandle">AccountHandle</a></td>
        <td></td>
      </tr>
</table>

#### monitor_account_addresses_balance()

Monitor account addresses for balance changes.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>account_handle</td>
        <td><a href="#accounthandle">AccountHandle</a></td>
        <td></td>
      </tr>
</table>

#### monitor_address_balance()

Monitor address for balance changes.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>account_handle</td>
        <td><a href="#accounthandle">AccountHandle</a></td>
        <td></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>addresses</td>
        <td><a href="#addresswrapper">AddressWrapper</a></td>
        <td></td>
      </tr>
</table>


## Migration

#### MigrationData

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>balance</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Total seed balance.</td>
      </tr>
  <tr>
        <td>last_checked_address_index</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>The index of the last checked address.</td>
      </tr>
  <tr>
        <td>inputs</td>
        <td>&#10004;</td>
        <td><a href="#inputdata">InputData</a></td>
        <td>Migration inputs.</td>
      </tr>
  <tr>
        <td>spent_addresses</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>If any of the inputs are spent</td>
      </tr>
</table>


## Signing

#### TransactionInput

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>input</td>
        <td>&#10004;</td>
        <td><a href="#input">Input</a></td>
        <td>The input.</td>
      </tr>
  <tr>
        <td>address_index</td>
        <td>&#10004;</td>
        <td>usize</td>
        <td>Input's address index.</td>
      </tr>
  <tr>
        <td>address_internal</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the input address is a change address or a public address.</td>
      </tr>
</table>


#### GenerateAddressMetadata

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>syncing</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Indicates that the address is being generated as part of the account syncing process.</td>
      </tr>
  <tr>
        <td>network</td>
        <td>&#10004;</td>
        <td><a href="#network">Network</a></td>
        <td>The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and</td>
      </tr>
</table>


## Storage

#### MessageIndexation

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>key</td>
        <td>&#10004;</td>
        <td><a href="#messageid">MessageId</a></td>
        <td>The message id.</td>
      </tr>
  <tr>
        <td>payload_hash</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>The payload hash.</td>
      </tr>
  <tr>
        <td>incoming</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the message is an incoming or an outgoing transaction.</td>
      </tr>
  <tr>
        <td>broadcasted</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the message was broadcasted or not.</td>
      </tr>
  <tr>
        <td>confirmed</td>
        <td>&#10004;</td>
        <td>bool</td>
        <td>Whether the message was confirmed or not.</td>
      </tr>
  <tr>
        <td>value</td>
        <td>&#10004;</td>
        <td>u64</td>
        <td>Message value.</td>
      </tr>
  <tr>
        <td>reattachment_message_id</td>
        <td>&#10004;</td>
        <td><a href="#messageid">MessageId</a></td>
        <td>Id of the message that reattached this message.</td>
      </tr>
</table>


## Syncing

### Account Syncing Process

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

#### AccountSynchronizer

<table>
      <tr>
        <td><strong>Property</strong></td>
        <td><strong>Required</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td><a href="#gap_limit()">gap_limit()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Number of address indexes that are generated.</td>
      </tr>
  <tr>
        <td><a href="#skip_persistence()">skip_persistence()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skip saving new messages and addresses on the account object.</td>
      </tr>
  <tr>
        <td><a href="#skip_change_addresses()">skip_change_addresses()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Skip syncing existing change addresses.</td>
      </tr>
  <tr>
        <td><a href="#address_index()">address_index()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Initial address index to start syncing.</td>
      </tr>
  <tr>
        <td><a href="#execute()">execute()</a></td>
        <td>&#10004;</td>
        <td>function</td>
        <td>Syncs account with the tangle.</td>
      </tr>
</table>

### AccountSynchronizer

#### gap_limit()

Number of address indexes that are generated.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>limit</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountsynchronizer">AccountSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### skip_persistence()

Skip saving new messages and addresses on the account object.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountsynchronizer">AccountSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### skip_change_addresses()

Skip syncing existing change addresses.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountsynchronizer">AccountSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### address_index()

Initial address index to start syncing.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>parameter</td>
        <td>address_index</td>
        <td>usize</td>
        <td></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#accountsynchronizer">AccountSynchronizer</a></td>
        <td></td>
      </tr>
</table>

#### execute()

Syncs account with the tangle.
<table>
      <tr>
        <td></td>
        <td><strong>Name</strong></td>
        <td><strong>Type</strong></td>
        <td><strong>Description</strong></td>
      </tr>
  <tr>
        <td>returns</td>
        <td></td>
        <td><a href="#syncedaccount">SyncedAccount</a></td>
        <td></td>
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


