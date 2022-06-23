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
import create_account from  '!!raw-loader!./../../../../bindings/nodejs/examples/1-create-account.js';

# Exchange Guide

## The IOTA Wallet Library

:::note

You can easily integrate IOTA with your exchange, custody solution, or product.

:::

## How Do I Implement It to My Exchange?

In [wallet.rs](../libraries/wallet.md), we use an account model so you can create an account for each of your users. Another approach would be to use one account and generate multiple addresses, which you can then link to the users in your database. The wallet library is designed to be as flexible as possible to back up any of your use cases.

Since addresses are reusable, they can be mapped to your users in a clear and concise way:

- Create an account for every user -> `Multi Account` approach.
- Create one account with many addresses -> `Single account` approach.

The library supports derivation for multiple accounts from a single seed. An account is simply a deterministic identifier from which multiple addresses can be further derived. 

The library also allows consumers to assign a meaningful alias to each account.

### Multi-Account Approach

The multi-account approach is used to create an account for each individual user. The created accounts can then be linked to the internal user IDs as an account alias, which are distinctly separated.

### Single Account Approach

The single account approach allows for just one account and creates addresses for each user. The associated addresses are then linked to the internal user IDs and store who owns which address in the database. Most exchanges are more familiar with the single account approach and find it easier to use, implement, and backup.

## Implementation Guide

This guide explains how to use the Wallet Library to successfully implement into an exchange.

Features of the Wallet Library:
- Secure seed management.
- Account management (with multiple accounts and multiple addresses).
- Confirmation monitoring.
- Deposit address monitoring.
- Backup and restore functionality.

## How Does it Work?

The Wallet Library is a stateful package with a standardized interface for developers to build applications involving value transactions. It offers abstractions to handle payments and can optionally interact with the IOTA Stronghold for seed handling, seed storage, and state backup.

For further reference, you can read our [wallet documentation here](https://wiki.iota.org/wallet.rs/welcome).

The following examples cover the *multi-account approach* using the `NodeJS` binding:

1. Set up the Wallet Library.
2. Create an account for each user.
3. Generate a User Address to Deposit Funds
4. Listen to events.
5. Check the user balance.
6. Enable withdrawals.

:::note

If you are looking for other languages, please read the [wallet library overview](../libraries/wallet).

:::

Since all `wallet.rs` bindings are based on core principles provided by the `wallet.rs` library, the outlined approach is very similar regardless of the programming language of your choice.

### 1. Set up the Wallet Library
First, you should install the components that are needed to use `wallet.rs` and the binding of your choice; it may vary a bit from language to language. In the case of the `NodeJs` binding, it is straightforward since it is distributed via the `npm` package manager. We also recommend you use `dotenv` for password management.

You can read more about [backup and security in this guide](./backup_security).

```bash
npm install @iota/wallet dotenv
touch .env
```

Then, input your password to the `.env` file like this:

```bash
SH_PASSWORD="here is your super secure password"
```

Once you have everything needed to use the `wallet.rs` library, it is necessary to initialize the `AccountManager` instance with a secret manager(`Stronghold` by default) and client options.

:::note

Manage your password with the utmost care.

:::

By default the Stronghold file will be called `wallet.stronghold`. It is needed to generate a seed (derived from the mnemonic) that serves as a cryptographic key from which all accounts and related addresses are generated.

One of the key principles behind the `stronghold` is that no one can get a seed out of it, so you should also backup the mnemonic (24 words) in a secure place. You deal with all the accounts purely via the `AccountManager` instance where all complexities are hidden under the hood and are dealt with securely.

:::note

Keep the `stronghold` password and the `stronghold` database on separate devices. See the [backup and security guide](./backup_security) for more information.

:::

Import the Wallet Library and create an account manager:

<CodeBlock className="language-javascript">
  {create_account}
</CodeBlock>

Once the stronghold storage is created, it is not needed to generate the seed any longer (`manager.storeMnemonic(SignerType.Stronghold, manager.generateMnemonic())`). It has already been saved in the storage together with all account information.

### 2. Create an Account For a User

Once the backend storage is created, individual accounts for individual users can be created:

```javascript
    let account = await manager.createAccount({
        alias: user_id,  // an unique id from your existing user
        clientOptions: { node: 'https://api.lb-0.h.chrysalis-devnet.iota.cafe/', localPow: false }
    })
```

Each account is related to a specific IOTA network (mainnet/devnet) which is referenced by a node property, such as node url (in this example, the Chrysalis devnet balancer).

For more information about `clientOptions`, please refer to the [Wallet NodeJs API Reference](https://wiki.iota.org/wallet.rs/libraries/nodejs/api_reference).

The `Alias` can be whatever fits to the given use case and needs to be unique. The `Alias` is typically used to identify the given account later on. Each account is also represented by an `index` which is incremented (by 1) every time a new account is created. Any account can then be referred to via `index`, `alias`.

Once an account has been created, you get an instance of it using `AccountManager.getAccount(accountId|alias)` or get all accounts with `AccountManager.getAccounts()`.

The most common methods of `account` instance include:

* `account.alias()` - returns an alias of the given account.
* `account.listAddresses()` - returns list of addresses related to the account.
* `account.generateAddress()` - generate a new address for the address index incremented by 1.
* `account.balance()` - returns the balance for the given account.
* `account.sync()` - sync the account information with the tangle.

### 3. Generate a User Address to Deposit Funds
`Wallet.rs` is a stateful library which means it caches all relevant information in storage to provide performance benefits while dealing with, potentially, many accounts/addresses.

Every account can have multiple addresses. Addresses are represented by an `index` which is incremented (by 1) every time a new address is created. The addresses are accessible via `account.listAddress()`: 

```javascript
    const addresses = account.listAddresses()

    console.log('Need a refill? Send it to this address:', addresses[0])
```
You can fill the address with Devnet Tokens with the [IOTA Faucet](https://faucet.devnet.chrysalis2.com/) to test it.

Addresses are of two types, `internal` and `public` (external):

* Each set of addresses are independent from each other and has an independent `index` id.
* Addresses that are created by `account.generateAddress()` are indicated as `internal=false` (public).
* Internal addresses (`internal=true`) are called `change` addresses and are used to send the excess funds to them.
* The approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

### 4. Listen to Events

The `Wallet.rs` library supports several events for listening. As soon as the given event occurs (which usually happens during syncing), a provided callback is triggered.

Below is an example for listening to new output events:

```javascript
    const callback = function(err, data) {
        if(err) console.log("err:", err)
         const event = JSON.parse(data)
        console.log("Event for account:", event.accountIndex)
        console.log("data:", event.event)
    }

    //Adds a new event handler for `NewOutput` with a callback in the form of (err, data) => {}.
    manager.listen(['NewOutput'], callback)

```

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

<!-- TODO -->
<!-- For further reference, you can read more about events in the [API reference](https://wiki.iota.org/wallet.rs/libraries/nodejs/api_reference#addeventlistenerevent-cb). -->

### 5. Check the Account Balance

Get the available account balance across all addresses of the given account:

```javascript
    // Sync account to get latest state from the network
    console.log('syncing...')
    const synced = await account.sync()
    console.log('synced!')
    let balance = account.balance().available
    console.log('available balance', balance)
```

### 6. Enable Withdrawals

Sending coins:

```javascript
    console.log('syncing...')
    const synced = await account.sync()
    console.log('available balance', account.balance().available)

    const address = 'atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r'
    const amount = 1000000 // Amount in IOTA: 1000000 == 1 MIOTA

    const transaction_result = await account.sendAmount([
            {
                address,
                amount,
            },
    ]);

    console.log("Check your message on https://explorer.iota.org/chrysalis/block/", transaction_result.id)
```

The full function signature is `Account.send(address, amount[, options])`.

Default options are fine and successful; however, additional options can be provided, such as `remainderValueStrategy`:

* `changeAddress`: Send the remainder value to an internal address.
* `reuseAddress`: Send the remainder value back to its original address.

```json
TransactionOptions {
    remainderValueStrategy?: RemainderValueStrategy;
    taggedDataPayload?: ITaggedDataPayload;
    skipSync?: boolean;
    customInputs?: string[];
}
```

The `Account.send()` function returns a `transaction` with it's id. The `blockId` can be used later for checking a confirmation status. Individual transactions related to the given account can be obtained via the `account.listTransactions()` function.

Please note that when sending tokens, a [dust protection](./developer#dust-protection) mechanism should be considered. 