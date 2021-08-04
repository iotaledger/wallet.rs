# Examples

This section will guide you through several examples using the node.js binding of the `wallet.rs` library. You can also find the code for the examples in the `/bindings/nodejs/examples` folder in the [official GitHub repository](https://github.com/iotaledger/wallet.rs/tree/dev/bindings/nodejs/examples).

All the examples in this section expect you to set your custom password  in the _.env_ file:

```bash
SH_PASSWORD="here is your super secure password"
```

## Account Manager and Individual Accounts
You can initialize (open) a secure storage for individual accounts.  The storage is backed up by `Stronghold` by default, using an AccountManager instance.  

The following example creates a new database and account:

```javascript
{@include: ../../../../bindings/nodejs/examples/1-create-account.js}
```

* Storage is initialized under the given path (`./alice-database`)
* The password is set based on your password in _.env_ file (`manager.setStrongholdPassword(process.env.SH_PASSWORD)` )
* When you initialize the new database, a Stronghold mnemonic (seed) is automatically generated and stored by default (`manager.storeMnemonic(SignerType.Stronghold)` ).
* The seed should be set only for the first time. In order to open already initialized database, you can simply use your password.

The storage is encrypted at rest, so you need a strong password and location where to place your storage. 

:::warning
We highly recommended that you to store your `Stronghold` password encrypted on rest and separated from `Stronghold` snapshots. 

Deal with the password with utmost care.
:::

 The storage comprises two things:
* A single file called _wallet.stronghold_ , which contains _seed_.  `Stronghold` will secure the seed and encrypt it at rest. The generated seed (mnemonic) serves as a cryptographic key, which is used to generate all accounts and related addresses.
* Other data used by library that is stored under _db_ sub-directory.  The includes account information, generated addresses, fetched messages, etc. This data is used to speed up some operations, such as account creation, address generation, etc.

One of the key principles behind `Stronghold` based storage is that no one can extract a seed from the storage. You deal with all accounts purely via an _AccountManager_ instance and all complexities are hidden under the hood and are dealt with securely.

If you also want to store a seed somewhere else, you can use the `AccountManager.generateMnemonic()` method. You can use this method to generate a random seed.  You can also use it before the actual account initialization.

You can find detailed information about seed generation at [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide#seed).

### Accounts

The `wallet.rs` library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically.  You can find more information about account management in the  [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/exchange_guide#how-do-i-implement-it-to-my-exchange).

Once the backend storage has been created, individual accounts for individual users can be created by running the `manager.createAccount()` method:

```javascript
    let account = await manager.createAccount({
        alias: 'Alice',  // an unique id from your existing user
        clientOptions: { node: 'http://api.lb-0.testnet.chrysalis2.com', localPow: false }
    })
```

Each account is related to a specific IOTA network (mainnet / testnet), which is referenced by a node properties such as node url.  In this example, the `Chrysalis` testnet balancer.

For more information about _clientOptions_ , please refer to [Wallet NodeJs API Reference](api_reference.md).

 _Alias_ should be unique, and it can be any string that you see fit. The _alias_ is usually used to identify the account later on. Each account is also represented by an _index_ which is incremented by 1 every time new account is created. 
Any account can be then referred to by its _index_ , _alias_ or one of its generated _addresses_ .

Several API calls can be performed via an _account_ instance.

:::info
It is a good practice to sync accounts with the Tangle every time you work with an _account_ instance.  This way you can ensure that you rely on the latest available information.

You can do this using `account.sync()`.`account.sync()` is performed automatically on `send`, `retry`,`reattach` and `promote` API calls.
:::

Once an account has been created, you can retrieve an instance using the following methods: 
- [`AccountManager.getAccount(accountId)` ](api_reference.md#getaccountaccountid)
- [`AccountManager.getAccountByAlias(alias)` ](api_reference.md#getaccountbyaliasalias)
- [`AccountManager.getAccounts()` .](api_reference.md#getaccounts)

The most common methods of _account_ instance are:
*`account.alias()` : returns an alias of the given account.
*`account.listAddresses()` : returns list of addresses related to the account.
*`account.getUnusedAddress()` : returns a first unused address.
*`account.generateAddress()` : generate a new address for the address index incremented by 1.
*`account.balance()` : returns the balance for the given account.
*`account.sync()` : sync the account information with the tangle.

## Generating Address(es)
Each account can have multiple addresses. Addresses are generated deterministically based on the account and address index. This means that the combination of account and index uniquely identifies the given address.

There are two types of addresses, _internal_ and _public_ (external), and each set of addresses is independent of each other and has independent _index_ id.

* _Public_ addresses are created by `account.generateAddress()` and  are indicated as _internal=false_ (public)
* _Internal_ addresses are also called `change` addresses. _Internal_ addresses are used to store the excess funds and are indicated as _internal=false_.

This approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

:::info
The IOTA 1.5 (Chrysalis) network supports reusing addresses multiple times.
::: 

You can use the following example to generate a new address:

```javascript
{@include: ../../../../bindings/nodejs/examples/2-generate-address.js}
```

## Checking Balance
Before we continue further, please visit the [IOTA testnet faucet service](https://faucet.testnet.chrysalis2.com/) and send to your testnet addresses some tokens.

![IOTA Faucet Service](/img/libraries/screenshot_faucet.png)

You can use the following example to generate a new database and account:

```javascript
{@include: ../../../../bindings/nodejs/examples/3-check_balance.js}
```

IOTA is based on _Unspent Transaction Output_ model. You can find a detailed explanation in the [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide#unspent-transaction-output-utxo).

## Sending tokens
You can use the following example to send tokens using an  Account  instance to any desired  address:

```javascript
{@include: ../../../../bindings/nodejs/examples/4-send.js}
```

The full function signature is `Account.send(address, amount, [options])`.
You can use the default options. However, you can provide additional options, such as `remainderValueStrategy` which has the following strategies:
*`changeAddress()` : Send the remainder value to an internal address
*`reuseAddress()` : Send the remainder value back to its original address

The `Account.send()` function returns a _wallet message_ that fully describes the given transaction. You can use the _messageId_ to check confirmation status. You can retrieve individual messages related to any given account using the `Account.listMessages()` function.

### Dust Protection

The network uses a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide#dust-protection) protocol to prevent malicious actors from spamming the network while also keeping track of the unspent amount ( _UTXO_ ).

:::info
Micro-transaction below 1Mi of IOTA tokens can be sent to another address if there is already at least 1Mi on that address. 
That's why we sent 1Mi in the last example, to comply with the dust protection.

Dust protection also means you can't leave less than 1Mi on a spent address (leave a dust behind).
:::

## Backup a database

Due to security practices that are incorporated in the Stronghold's DNA, there is no way to retrieve a seed, as it is encrypted at rest.  Therefore, if you're using the default options, you should make sure that you back up your seed regularly. 

The following example will guide you in backing up your data in secure files. You can move this file to another app or device, and restore it.

```javascript
{@include: ../../../../bindings/nodejs/examples/5-backup.js}
```

Alternatively, you can create a copy of the _wallet.stronghold_ file and use it as seed backup. This can be achieved by a daily [_cronjob_](https://linux.die.net/man/1/crontab), [_rsync_](https://linux.die.net/man/1/rsync) or [_scp_](https://linux.die.net/man/1/scp) with a datetime suffix for example.

## Restore a Database

To restore a database via `wallet.rs`, you will need to create new empty database with a password (without mnemonic seed).  After you've created the empty database, you will need to import all accounts from the file that has been backed up earlier

The following example restores a secured backup file:

```javascript
{@include: ../../../../bindings/nodejs/examples/6-restore.js}
```

Since the backup file is just a copy of the original database it can be also be renamed to _wallet.stronghold_ and opened in a standard way.

## Listening to events

`wallet.rs` library is able to listen to several supported event. As soon as the event occurs, a provided callback will be triggered.

You can use the following example to fetch an existing _Account_ and listen to transaction events related to that _Account_ :

```javascript
{@include: ../../../../bindings/nodejs/examples/7-events.js}
```

Example output:

```json
data: {
  accountId: 'wallet-account://1666fc60fc95534090728a345cc5a861301428f68a237bea2b5ba0c844988566',
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

You can then use the _accountId_ to identify the account via `AccountManager.getAccount(accountId)`.

Read more about Events in the [API reference](api_reference.md#addeventlistenerevent-cb).

## Migration 

You can use the following example to create a new database and account, and migrate funds from the legacy network to the `Chrysalis` network.

Run:
```
node 8-migration.js
```

Code:

```javascript
{@include: ../../../../bindings/nodejs/examples/8-migration.js}
```