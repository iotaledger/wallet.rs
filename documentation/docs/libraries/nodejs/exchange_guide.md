---
description: Easily integrate with your exchange, custody solution, or product using the wallet.rs library.
keywords:
- integrate
- exchange
- account model
- addresses
- wallet.rs
- setup
- NodeJS
- explanation
---

import CodeBlock from '@theme/CodeBlock';
import generate_mnemonic from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/0-generate-mnemonic.js';
import create_account from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/1-create-account.js';
import generate_address from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/2-generate-address.js';
import check_balance from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/3-check-balance.js';
import listen_events from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/4-listen-events.js';
import send_amount from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/5-send-amount.js';

# Stardust Exchange Integration Guide


:::note

You can easily integrate wallet.rs with your exchange, custody solution, or product.

:::

## Account Approaches

In [wallet.rs](../../welcome.md), you can use an account model to [create an account for each user](#multi-account-approach) or [use one account and generate multiple addresses](#single-account-approach), which you can then link to the users in your database. The wallet library is as flexible as possible and can back up any of your use cases.

The library supports derivation for multiple accounts from a single seed. An account is simply a deterministic identifier from which multiple addresses can be further derived. 

The library also allows consumers to assign a meaningful alias to each account. Since addresses are reusable, they can be mapped to your users in a clear and concise way.

### Multi-Account Approach

You should use the multi-account approach if you want to create an account for each individual user. You can link the accounts to the internal user IDs as an account alias, which are distinctly separated.

### Single Account Approach

You should use the single account approach if you want to create a single account and then create an address for each user. You will need to link the associated addresses to the internal user IDs and store who owns which address in a database. Most exchanges are familiar with the single account approach and find it easier to use, implement, and backup.

## Implementation Guide

This guide explains how to implement the Wallet library in your exchange.

Features of the Wallet Library:
- Secure seed management.
- Account management with multiple accounts and multiple addresses.
- Confirmation monitoring.
- Deposit address monitoring.
- Backup and restore functionality.

### How Does it Work?

The Wallet Library is a stateful package with a standardized interface for developers to build applications involving value transactions. It offers abstractions to handle payments and can optionally interact with [Stronghold](https://wiki.iota.org/stronghold.rs/getting_started) for seed handling, seed storage, and state backup.

:::note

If you are not familiar with the wallet.rs library, you can find more information in the [documentation](../../welcome.md).

:::

You can use the following examples as a guide to implementing *the multi-account approach* using the `NodeJS` binding:

1. [Set up the wallet.rs library](#1-set-up-the-wallet-library).
2. [Create an account for each user](#2-create-an-account-for-a-user).
3. [Generate a user address to deposit funds](#3-generate-a-user-address-to-deposit-funds).
4. [Check the user balance](#4-check-the-account-balance).
5. [Listen to events](#5-listen-to-events).
6. [Enable withdrawals](#6-enable-withdrawals).

:::note

If you are looking for other languages, please read the [wallet.rs library overview](https://wiki.iota.org/wallet.rs/develop/overview).

:::

Since all `wallet.rs` bindings are based on core principles provided by the `wallet.rs` library, the outlined approach is very similar regardless of the programming language you choose.

### 1. Set Up the Wallet.rs Library

First, you should install the components that are needed to use `wallet.rs` and the binding of your choice; it may vary a bit from language to language. In the case of the [NodeJs binding](./getting_started.md), it is straightforward since it is distributed via the `npm` package manager.

You can read more about [backup and security in this guide](https://wiki.iota.org/introduction/guides/backup_security).

```bash
npm install @iota/wallet dotenv
```

#### 1 Generate a mnemonic

<CodeBlock className="language-javascript">
  {generate_mnemonic}
</CodeBlock>

You can then create a `.env` by running the following command:

```bash
touch .env
```

You can now add your `SH_PASSWORD` and `MNEMONIC` to the `.env` file. 

```bash
SH_PASSWORD="here is your super secure password"
MNEMONIC="here is your super secure 24 word mnemonic, it's only needed here the first time"
```

After you have updated the `.env` file, you can initialize the `AccountManager` instance with a secret manager([`Stronghold`](https://wiki.iota.org/stronghold.rs/getting_started) by default) and client options.

:::note

Manage your password with the utmost care.

:::

By default, the Stronghold file will be called `wallet.stronghold`. It will store the seed (derived from the mnemonic) that serves as a cryptographic key from which all accounts and related addresses are generated.

One of the key principles behind the `stronghold` is that no one can get a seed out of it, so you should also back up your 24-word mnemonic in a secure place because there is no way to recover it from the `.stronghold` file. You deal with accounts using the `AccountManager` instance exclusively, where all complexities are hidden under the hood and are dealt with securely.

:::note

Keep the `stronghold` password and the `stronghold` database on separate devices. See the [backup and security guide](https://wiki.iota.org/introduction/guides/backup_security) for more information.

:::

#### 2 Create an account

You can import the Wallet Library and create an account manager using the following example:

<CodeBlock className="language-javascript">
  {create_account}
</CodeBlock>

The `Alias` must be unique and can be whatever fits your use case. The `Alias` is typically used to identify an account later on. Each account is also represented by an `index` which is incremented by one every time a new account is created. You can refer to any account via its `index`, or `alias`.

You get an instance of any created account using `AccountManager.getAccount(accountId|alias)` or get all accounts with `AccountManager.getAccounts()`.

Common methods of `account` instance include:

* `account.listAddresses()` - returns list of addresses related to the account.
* `account.generateAddress()` - generate a new address for the address index incremented by 1.
* `account.balance()` - returns the balance for the given account.
* `account.sync()` - sync the account information with the tangle.

### 3. Generate a User Address to Deposit Funds

`Wallet.rs` is a stateful library. This means it caches all relevant information in storage to provide performance benefits while dealing with, potentially, many accounts and addresses.
<CodeBlock className="language-javascript">
  {generate_address}
</CodeBlock>

Every account can have multiple addresses. Addresses are represented by an `index` which is incremented by one every time a new address is created. You can access the addresses using the `account.listAddress()` method: 

```javascript
    const addresses = account.listAddresses()

    console.log('Need a refill? Send it to this address:', addresses[0])
```

You can use the [Faucet](https://faucet.testnet.shimmer.network/) to add test tokens and test your account.

There are two types of addresses, `internal` and `public` (external). This approach is known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*. 

* Each set of addresses is independent of each other and has an independent `index` id.
* Addresses that are created by `account.generateAddress()` are indicated as `internal=false` (public).
* Internal addresses (`internal=true`) are called `change` addresses and are used to send the excess funds to them.

### 4. Check the Account Balance

:::warning

Outputs can have multiple [unlock conditions](https://github.com/lzpap/tips/blob/master/tips/TIP-0018/tip-0018.md#unlock-conditions), which could require one to send some or the full amount back, which could expire if not claimed in time or which might not be unlockable for a very long time.
To get only outputs with the AddressUnlockCondition alone, that don't need extra checks for the ownership, sync with `syncOnlyMostBasicOutputs: true`. When syncing also other outputs, the unlock conditions must be carefully checked before crediting users any balance.

:::

You can get the available account balance across all addresses of the given account using the following example:

<CodeBlock className="language-javascript">
  {check_balance}
</CodeBlock>

### 5. Listen to Events

:::warning

Outputs can have multiple [unlock conditions](https://github.com/lzpap/tips/blob/master/tips/TIP-0018/tip-0018.md#unlock-conditions), which could require one to send some or the full amount back, which could expire if not claimed in time or which might not be unlockable for a very long time.
To get only outputs with the AddressUnlockCondition alone, that don't need extra checks for the ownership, sync with `syncOnlyMostBasicOutputs: true`. When syncing also other outputs, the unlock conditions must be carefully checked before crediting users any balance.

:::

The `Wallet.rs` library supports several events for listening. A provided callback is triggered as soon as an event occurs (which usually happens during syncing).

You can use the following example to listen to new output events:

<CodeBlock className="language-javascript">
  {listen_events}
</CodeBlock>

**Example output:**

```json
NewOutput: {
  output: {
    outputId: '0x2df0120a5e0ff2b941ec72dff3464a5b2c3ad8a0c96fe4c87243e4425b9a3fe30000',
    metadata: [Object],
    output: [Object],
    isSpent: false,
    address: [Object],
    networkId: '1862946857608115868',
    remainder: false,
    chain: [Array]
  },
  transaction: null,
  transactionInputs: null
}
```

Alternatively you can use `account.listOutputs()` to get all outputs that are stored in the account, or `account.listUnspentOutputs()`, to get only unspent outputs.

### 6. Enable Withdrawals

You can use the following example to send tokens to an address.

<CodeBlock className="language-javascript">
  {send_amount}
</CodeBlock>

The full function signature is `account.sendAmount(outputs[, options])`.

Default options are fine and successful; however, you can provide additional options, such as `remainderValueStrategy`, which can have the following values:

* `changeAddress`: Send the remainder value to an internal address.
* `reuseAddress`: Send the remainder value back to its original address.
* `customAddress`: Send the remainder value back to a provided account address.

```json
TransactionOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    customInputs?: string[];
}
```

The `account.sendAmount()` function returns a `transaction` with it's id. The `blockId` can be used later for checking a confirmation status. You can obtain individual transactions related to the given account using the `account.listTransactions()` function.

:::note  Dust Protection

When sending tokens, you should consider a [dust protection](https://github.com/muXxer/tips/blob/master/tips/TIP-0019/tip-0019.md) mechanism.
  
:::