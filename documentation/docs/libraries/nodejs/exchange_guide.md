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
import generate_mnemonic from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/1-generate-mnemonic.js';
import create_account from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/2-create-account.js';
import generate_address from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/3-generate-address.js';
import check_balance from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/4-check-balance.js';
import listen_events from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/5-listen-events.js';
import send_amount from  '!!raw-loader!./../../../../bindings/nodejs/examples/exchange/6-send-amount.js';

# Stardust Exchange Integration Guide

## The Wallet Library

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

This guide explains how to use the Wallet Library to successfully implement into an exchange.

Features of the Wallet Library:
- Secure seed management.
- Account management (with multiple accounts and multiple addresses).
- Confirmation monitoring.
- Deposit address monitoring.
- Backup and restore functionality.

## How Does it Work?

The Wallet Library is a stateful package with a standardized interface for developers to build applications involving value transactions. It offers abstractions to handle payments and can optionally interact with Stronghold for seed handling, seed storage, and state backup.

For further reference, you can read our [wallet documentation here](https://wiki.iota.org/wallet.rs/welcome).

The following examples cover the *multi-account approach* using the `NodeJS` binding:

1. Set up the Wallet Library.
2. Create an account for each user.
3. Generate a User Address to Deposit Funds
4. Listen to events.
5. Check the user balance.
6. Enable withdrawals.

:::note

If you are looking for other languages, please read the [wallet library overview](https://wiki.iota.org/wallet.rs/welcome).

:::

Since all `wallet.rs` bindings are based on core principles provided by the `wallet.rs` library, the outlined approach is very similar regardless of the programming language of your choice.

### 1. Set up the Wallet Library
First, you should install the components that are needed to use `wallet.rs` and the binding of your choice; it may vary a bit from language to language. In the case of the `NodeJs` binding, it is straightforward since it is distributed via the `npm` package manager. We also recommend you use `dotenv` for password management.

You can read more about [backup and security in this guide](https://wiki.iota.org/introduction/guides/backup_security).

```bash
npm install @iota/wallet dotenv
```

#### 1.1 Generate a mnemonic

<CodeBlock className="language-javascript">
  {generate_mnemonic}
</CodeBlock>

Then, input your password to the `.env` file like this:

```bash
touch .env
```

```bash
SH_PASSWORD="here is your super secure password"
MNEMONIC="here is your super secure 24 word mnemonic, it's only needed here the first time"
```

Once you have everything needed to use the `wallet.rs` library, it is necessary to initialize the `AccountManager` instance with a secret manager(`Stronghold` by default) and client options.

:::note

Manage your password with the utmost care.

:::

By default the Stronghold file will be called `wallet.stronghold`. It will store the seed (derived from the mnemonic) that serves as a cryptographic key from which all accounts and related addresses are generated.

One of the key principles behind the `stronghold` is that no one can get a seed out of it, so you should also backup the mnemonic (24 words) in a secure place, because there is no way to recover it from the `.stronghold` file. You deal with all the accounts purely via the `AccountManager` instance where all complexities are hidden under the hood and are dealt with securely.

:::note

Keep the `stronghold` password and the `stronghold` database on separate devices. See the [backup and security guide](https://wiki.iota.org/introduction/guides/backup_security) for more information.

:::

#### 1.2 Create an account

Import the Wallet Library and create an account manager:

<CodeBlock className="language-javascript">
  {create_account}
</CodeBlock>

### 2. Create an Account For a User

Once the backend storage is created, individual accounts for individual users can be created.

The `Alias` can be whatever fits to the given use case and needs to be unique. The `Alias` is typically used to identify the given account later on. Each account is also represented by an `index` which is incremented (by 1) every time a new account is created. Any account can then be referred to via `index`, `alias`.

Once an account has been created, you get an instance of it using `AccountManager.getAccount(accountId|alias)` or get all accounts with `AccountManager.getAccounts()`.

The most common methods of `account` instance include:

* `account.listAddresses()` - returns list of addresses related to the account.
* `account.generateAddress()` - generate a new address for the address index incremented by 1.
* `account.balance()` - returns the balance for the given account.
* `account.sync()` - sync the account information with the tangle.

### 3. Generate a User Address to Deposit Funds
`Wallet.rs` is a stateful library which means it caches all relevant information in storage to provide performance benefits while dealing with, potentially, many accounts/addresses.

<CodeBlock className="language-javascript">
  {generate_address}
</CodeBlock>

Every account can have multiple addresses. Addresses are represented by an `index` which is incremented (by 1) every time a new address is created. The addresses are accessible via `account.listAddress()`: 

```javascript
    const addresses = account.listAddresses()

    console.log('Need a refill? Send it to this address:', addresses[0])
```
You can fill the address with Tokens with the [Faucet](https://faucet.testnet.shimmer.network/) to test it.

Addresses are of two types, `internal` and `public` (external):

* Each set of addresses are independent from each other and has an independent `index` id.
* Addresses that are created by `account.generateAddress()` are indicated as `internal=false` (public).
* Internal addresses (`internal=true`) are called `change` addresses and are used to send the excess funds to them.
* The approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

### 4. Check the Account Balance

Get the available account balance across all addresses of the given account:

<CodeBlock className="language-javascript">
  {check_balance}
</CodeBlock>

### 5. Listen to Events

The `Wallet.rs` library supports several events for listening. As soon as the given event occurs (which usually happens during syncing), a provided callback is triggered.

Below is an example for listening to new output events:

<CodeBlock className="language-javascript">
  {listen_events}
</CodeBlock>

Example output:

```bash
TODO: update
data: {
  accountId: '0',
  address: {
    address: 'atoi1q9c6r2ek5w2yz54en78m8dxwl4qmwd7gmh9u0krm45p8txxyhtfry6apvwj',
    balance: 20000000,
    keyIndex: 0,
    internal: false,
    outputs: [ [Object], [Object] ]
  },
  balance: 20000000
}
```

`accountId` can then be used to identify the given account via `AccountManager.getAccount(accountId)`.

### 6. Enable Withdrawals

You can use the following example to send tokens to an address.

<CodeBlock className="language-javascript">
  {send_amount}
</CodeBlock>

The full function signature is `Account.sendAmount(outputs[, options])`.

Default options are fine and successful; however, you can provide additional options, such as `remainderValueStrategy`, which can have the following values:

* `changeAddress`: Send the remainder value to an internal address.
* `reuseAddress`: Send the remainder value back to its original address.

```json
TransactionOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    customInputs?: string[];
}
```

The `Account.sendAmount()` function returns a `transaction` with it's id. The `blockId` can be used later for checking a confirmation status. You can obtain individual transactions related to the given account using the `account.listTransactions()` function.

:::note  Dust Protection

When sending tokens, you should consider a [dust protection](https://github.com/muXxer/tips/blob/master/tips/TIP-0019/tip-0019.md) mechanism.
  
:::