# Examples

There are several examples to show the usage of the library. All examples below can be also found in [/bindings/nodejs/examples](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/nodejs/examples)

All examples below expect your custom password in the `.env` file:
```bash
SH_PASSWORD="here is your super secure password"
```
> Please note: In is not recommended to store passwords on host's environment variables or in the source code in a production setup! Please make sure you follow our [backup and security](https://chrysalis.docs.iota.org/guides/backup_security.html) recommendations for production use!

## Account manager and individual accounts
First of all, let's initialize (open) a secure storage for individual accounts (backed up by Stronghold by default) using `AccountManager` instance:
```javascript
{{ #include ../../../bindings/nodejs/examples/1-create-account.js }}
```
* Storage is initialized under the given path (`./alice-database`)
* Password is set based on your password in `.env` file (`manager.setStrongholdPassword(process.env.SH_PASSWORD)`)
* Only during the initialization new database: stronghold mnemonic (seed) is automatically generated and stored by default (`manager.storeMnemonic(SignerType.Stronghold)`)
* Needless to say, the seed should be set only for the first time. In order to open already initialized database, just a password is enough

The storage is encrypted at rest and so you need a strong password and location where to put your storage. Please note: it is highly recommended to store `stronghold` password encrypted on rest and separated from `stronghold` snapshots.

> Please note: deal with the password with utmost care

Technically speaking, the storage means a single file called `wallet.stronghold`. The generated seed (mnemonic) serves as a cryptographic key from which all accounts and related addresses are generated.

One of the key principle behind the `stronghold`-based storage is that no one can get a seed from the storage. You deal with all accounts purely via `AccountManager` instance and all complexities are hidden under the hood and are dealt with in a secure way.

In case one would like to store a seed also somewhere else, there is a method `AccountManager.generateMnemonic()` that generates random seed and it can be leveraged before the actual account initialization.

### Accounts
The library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically. 

Once the backend storage is created, individual accounts for individual users can be created:
```javascript
    let account = await manager.createAccount({
        alias: 'Alice',  // an unique id from your existing user
        clientOptions: { node: 'http://api.lb-0.testnet.chrysalis2.com', localPow: false }
    })
```
Each account is related to a specific IOTA network (mainnet / devnet) which is referenced by a node properties, such as node url (in this example, Chrysalis testnet balancer).

For more information about `clientOptions`, please refer to [Wallet NodeJs API Reference](https://wallet-lib.docs.iota.org/libraries/nodejs/api_reference.html).

`Alias` can be whatever fits to the given use case and should be unique. The `alias` is usually used to identify the given account later on. Each account is also represented by `index` which is incremented (by 1) every time new account is created. 
Any account can be then referred to via `index`, `alias` or one of its generated `addresses`.

Several api calls can be performed via `account` instance.

> Note: it is a good practice to sync the given account with the Tangle every time you work with `account` instance to rely on the latest information available: `account.sync()`.

Once an account has been created you get an instance of it using the following methods: `AccountManager.getAccount(accountId)`, `AccountManager.getAccountByAlias(alias)` or `AccountManager.getAccounts()`.

The most common methods of `account` instance:
* `account.alias()`: returns an alias of the given account
* `account.listAddresses()`: returns list of addresses related to the account
* `account.getUnusedAddress()`: returns a first unused address
* `account.generateAddress()`: generate a new address for the address index incremented by 1
* `account.balance()`: returns the balance for the given account
* `account.sync()`: sync the account information with the tangle

## Generating address(es)
Each account can posses multiple addresses. Addresses are generated deterministically based on the account and address index. It means that the combination of account and index uniquely identifies the given address.

Addresses are of two types: `internal` and `public` (external):
* each set of addresses is independent from each other and has independent `index` id
* addresses that are created by `account.generateAddress()` are indicated as `internal=false` (public)
* internal addresses (`internal=true`) are so called `change` addresses and are used to send the excess funds to
* the approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

_Note: You may remember IOTA 1.0 network in which addresses were not reusable. It is no longer true and addresses can be reused multiple times in IOTA 1.5 (Chrysalis) network._

```javascript
{{ #include ../../../bindings/nodejs/examples/2-generate-address.js }}
```

## Checking balance
Before we continue further, go to [IOTA testnet faucet service](https://faucet.testnet.chrysalis2.com/) and send to your testnet addresses some tokens.

```javascript
{{ #include ../../../bindings/nodejs/examples/3-check_balance.js }}
```

## Sending tokens
Sending tokens is performed via `SyncedAccount` instance that is a results of `account.sync()` function:

```javascript
{{ #include ../../../bindings/nodejs/examples/4-send.js }}
```
The full function signature is `SyncedAccount.send(address, amount[, options])`.
Default options are perfectly fine and do the job done, however additional options can be provided, such as `remainderValueStrategy`:
* `changeAddress`: Send the remainder value to an internal address
* `reuseAddress`: Send the remainder value back to its original address

`SyncedAccount.send()` function returns a `wallet message` that fully describes the given transaction. Especially `messageId` can later be used for checking a confirmation status. Individual messages related to the given account can be obtained via `account.listMessages()` function.

### Dust protection
Please note, there is also implemented a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide.html#dust-protection) mechanism in the network protocol to avoid malicious actors to spam network in order to decrease node performance while keeping track of unspent amount (`UTXO`):
> "... microtransaction below 1Mi of IOTA tokens [can be sent] to another address if there is already at least 1Mi on that address"
That's why we did send 1Mi in the given example to comply with the protection."

## Backup database
Underlying database (provided by `Stronghold` by default) is encrypted at rest and there is no way how to get a seed from it due to security practices that are incorporated in the Stronghold's DNA. It means you are dealing with the database as an atomic unit that includes all wallet information.

So backing up the database is very important task from this respect:
```javascript
{{ #include ../../../bindings/nodejs/examples/5-backup.js }}
```
Alternatively, a simple copy of the `wallet.stronghold` file works as a backup. (e.g. a daily cronjob rsync / scp with a datetime suffix for example).

## Restore database
The process of restoring underlying database via `wallet.rs` can be described as follows:
* create new empty database with a password (without mnemonic [seed])
* import all accounts from the file that has been backed up earlier

```javascript
{{ #include ../../../bindings/nodejs/examples/6-restore.js }}
```

Since the backup file is just a copy of the original database it can be alternatively also renamed to `wallet.stronghold` and opened in a standard way.

## Listening to events
`Wallet.rs` library supports several events to be listened to. As soon as the given even occurs, a provided callback is triggered.

Example of fetching existing accounts and listen to transaction events coming into the account:
```javascript
{{ #include ../../../bindings/nodejs/examples/7-events.js }}
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

`accountId` can then be used to identify the given account via `AccountManager.getAccount(accountId)`.

Read more about Events in the [API reference](https://wallet-lib.docs.iota.org/libraries/nodejs/api_reference.html#addeventlistenerevent-cb).
