# Examples

This section will guide you through several examples using the node.js binding of the `Wallet.rs` library. You can also find the code for the examples in the `/bindings/nodejs/examples` folder in the [official GitHub repository](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/nodejs/examples).

All the examples in this section expect you to set your custom password  in the _.env_ file:
```bash
SH_PASSWORD="here is your super secure password"
```

## Account Manager and Individual Accounts
You can initialize (open) a secure storage for individual accounts.  The storage is backed up by `Stronghold` by default, using an AccountManager instance.  

The following example creates a new database and account:
```javascript
require('dotenv').config()

async function run() {
    const { AccountManager, SignerType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database',
    })
    manager.setStrongholdPassword(process.env.SH_PASSWORD)
    manager.storeMnemonic(SignerType.Stronghold)

    const account = await manager.createAccount({
        clientOptions: { node: "https://api.lb-0.testnet.chrysalis2.com", localPow: true },
        alias: 'Alice',
    })

    console.log('Account created:', account.alias())
      
}

run()
```
* Storage is initialized under the given path (`./alice-database`)
* The password is set based on your password in _.env_ file ( _manager.setStrongholdPassword(process.env.SH_PASSWORD)_ )
* When you initialize the new database, a Stronghold mnemonic (seed) is automatically generated and stored by default ( _manager.storeMnemonic(SignerType.Stronghold)_ ).
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

If you also want to store a seed somewhere else, you can use the _AccountManager.generateMnemonic()_ method. You can use this method to generate a random seed.  You can also use it before the actual account initialization.

You can find detailed information about seed generation at [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#seed).

### Accounts
The `Wallet.rs` library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically. 

Once the backend storage has been created, individual accounts for individual users can be created by running the _manager.createAccount()_ method:

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

You can do this using _account.sync()_. _account.sync()_ is performed automatically on _send_, _retry_, _reattach_ and _promote_ API calls.
:::

Once an account has been created, you can retrieve an instance using the following methods: 
- [ _AccountManager.getAccount(accountId)_ ](api_reference.md#getaccountaccountid)
- [ _AccountManager.getAccountByAlias(alias)_ ](api_reference.md#getaccountbyaliasalias)
- [ _AccountManager.getAccounts()_ .](api_reference.md#getaccounts)

The most common methods of _account_ instance are:
* _account.alias()_ : returns an alias of the given account.
* _account.listAddresses()_ : returns list of addresses related to the account.
* _account.getUnusedAddress()_ : returns a first unused address.
* _account.generateAddress()_ : generate a new address for the address index incremented by 1.
* _account.balance()_ : returns the balance for the given account.
* _account.sync()_ : sync the account information with the tangle.

## Generating Address(es)
Each account can have multiple addresses. Addresses are generated deterministically based on the account and address index. This means that the combination of account and index uniquely identifies the given address.

There are two types of addresses, _internal_ and _public_ (external), and each set of addresses is independent of each other and has independent _index_ id.

* _Public_ addresses are created by _account.generateAddress()_ and  are indicated as _internal=false_ (public)
* _Internal_ addresses are also called _change_ addresses. _Internal_ addresses are used to store the excess funds and are indicated as _internal=false_.

This approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

:::info
The IOTA 1.5 (Chrysalis) network supports reusing addresses multiple times.
::: 

You can use the following example to generate a new address:
```javascript
require('dotenv').config()

async function run() {
	const { AccountManager } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())

    // Always sync before doing anything with the account
    const synced = await account.sync()
    console.log('Syncing...')

    const { address } = account.generateAddress()
    console.log('New address:', address)

    // You can also get the latest unused address:
    // const addressObject = account.latestAddress()
    // console.log("Address:", addressObject.address)

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log("Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/")
}

run()
```

## Checking Balance
Before we continue further, please visit the [IOTA testnet faucet service](https://faucet.testnet.chrysalis2.com/) and send to your testnet addresses some tokens.

![IOTA Faucet Service](../../../static/img/libraries/screenshot_faucet.png)

You can use the following example to generate a new database and account:

```javascript

require('dotenv').config()

async function run() {
	const { AccountManager } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    
    console.log('Account:', account.alias())
    
    // Always sync before doing anything with the account
    const synced = await account.sync()
    console.log('Syncing...')

    console.log('Available balance', account.balance().available)
}

run()
```
IOTA is based on _Unspent Transaction Output_ model. You can find a detailed explanation in the [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#unspent-transaction-output-utxo).

## Sending tokens
You can use the following example to send tokens using an  Account  instance to any desired  address:

```javascript
 require('dotenv').config();

async function run() {
	const { AccountManager, RemainderValueStrategy } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    
    console.log('alias', account.alias())
    console.log('syncing...')
    const synced = await account.sync()
    console.log('available balance', account.balance().available)
    
    //TODO: Replace with the address of your choice!
	const addr = 'atoi1qykf7rrdjzhgynfkw6z7360avhaaywf5a4vtyvvk6a06gcv5y7sksu7n5cs'
	const amount = 10000000

	const node_response = await account.send(
		addr,
		amount,
        {remainderValueStrategy: RemainderValueStrategy.reuseAddress()}
    ) 

    console.log(`Check your message on https://explorer.iota.org/chrysalis/message/${node_response.id}`)
}

run()
```
The full function signature is `Account.send(address, amount, [options])`.
You can use the default options. However, you can provide additional options, such as _remainderValueStrategy_ which has the following strategies:
* _changeAddress()_ : Send the remainder value to an internal address
* _reuseAddress()_ : Send the remainder value back to its original address

The _Account.send()_ function returns a _wallet message_ that fully describes the given transaction. You can use the _messageId_ to check confirmation status. You can retrieve individual messages related to any given account using the _Account.listMessages()_ function.

### Dust protection
The network uses a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide.html#dust-protection) protocol to prevent malicious actors from spamming the network while also keeping track of the unspent amount ( _UTXO_ ).

:::info
Micro-transaction below 1Mi of IOTA tokens can be sent to another address if there is already at least 1Mi on that address. 
That's why we sent 1Mi in the last example to comply with the dust protection.
:::

Dust protection also means you can't leave less than 1Mi on a spent address (leave a dust behind).

## Backup a database

Due to security practices that are incorporated in the `Stronghold's` DNA, there's no way to retrieve a seed, as it is encrypted at rest.  Therefore, if you're using the default options,  backing up the seed storage is a very important task. 

The following example will guide you in backing up your data in secure files. You can move this file to another app or device, and restore it.

```javascript
require('dotenv').config();

async function run() {

    const { AccountManager } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    let backup_path = await manager.backup("./backup", process.env.SH_PASSWORD)
    
    console.log('Backup path:', backup_path)
}

run()
```
Alternatively, you can create a copy of the _wallet.stronghold_ file and use it as seed backup. This can be achieved by a daily [_cronjob_](https://linux.die.net/man/1/crontab), [_rsync_](https://linux.die.net/man/1/rsync) or [_scp_](https://linux.die.net/man/1/scp) with a datetime suffix for example.

## Restore database
To restore a database via `Wallet.rs`, you will need to:
1. Create new empty database with a password (without mnemonic seed)
2. Import all accounts from the file that has been backed up earlier

The following example restores a secured backup file:
```javascript
require('dotenv').config();

async function run() {

    const { AccountManager } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    // Add the path to the file from example 5-backup.js
    // for example: ./backup/2021-02-12T01-23-11-iota-wallet-backup-wallet.stronghold
    let backup_path = "input your backup file"

    await manager.importAccounts(backup_path, process.env.SH_PASSWORD)
    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())
}

run()
```

Since the backup file is just a copy of the original database it can be also be renamed to _wallet.stronghold_ and opened in a standard way.

## Listening to events
`Wallet.rs` library is able to listen to several supported event. As soon as the event occurs, a provided callback will be triggered.

You can use the following example to fetch an existing _Account_ and listen to transaction events related to that _Account_ :
```javascript

require('dotenv').config()

async function run() {
    const { AccountManager, addEventListener } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database'
    })

    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    const account = manager.getAccount('Alice')
    console.log('Account:', account.alias())

    // Always sync before doing anything with the account
    const synced = await account.sync()
    console.log('Syncing...')
    // let address = account.generateAddress()

    // get latest address
    let addressObject = account.latestAddress()

    console.log("Address:", addressObject.address)

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log("Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/")


    const callback = function (err, data) {
        console.log("data:", data)
    }

    addEventListener("BalanceChange", callback)

    // Possible Event Types:
    //
    // ErrorThrown
    // BalanceChange
    // NewTransaction
    // ConfirmationStateChange
    // Reattachment
    // Broadcast
}

run()
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

You can then use the _accountId_ to identify the account via _AccountManager.getAccount(accountId)_.

Read more about Events in the [API reference](api_reference.md#addeventlistenerevent-cb).


## Migration 
You can use the following example to create a new database and account, and migrate funds from the legacy network to the `Chrysalis` network.

Run:
```
node 8-migration.js
```

Code:
```javascript
require('dotenv').config()

// Address security level
const ADDRESS_SECURITY_LEVEL = 2
// Minimum balance that is required for a migration bundle, because of the dust protection in the new network
const MINIMUM_MIGRATION_BALANCE = 1000000
// This value shouldn't be too high, because then the PoW could take to long to get it confirmed
const MAX_INPUTS_PER_BUNDLE = 10
// Wallet.rs database storage path. Stronghold and database file would be stored in this path.
const DB_STORAGE_PATH = './migration-database'
// Legacy network nodes
const LEGACY_NETWORK_NODES = ['https://nodes.iota.org']
// Legacy permanode
const LEGACY_PERMANODE = 'https://chronicle.iota.org/api'
// Chrysalis node
const CHRYSALIS_NODE = 'https://chrysalis-nodes.iota.cafe'

async function run() {
  try {
    const { AccountManager, SignerType, addEventListener } = require('@iota/wallet')

    // We store all bundle hashes here and check later if the bundles got confirmed
    let migrationBundleHashes = [];

    // Log migration events
    const callback = function (err, data) {
      // After a successful broadcast of this bundle, the library will automatically reattach bundle to 
      // speed up the confirmation process. An event with type "TransactionConfirmed" (with corresponding bundle hash) 
      // will be emitted as soon as the bundle is confirmed.
      if (data.event.type === 'TransactionConfirmed') {
        console.log("MigrationProgress:", data)
        migrationBundleHashes = migrationBundleHashes.filter(hash => hash !== data.event.data.bundleHash)

        if (migrationBundleHashes.length == 0) {
          process.exit()
        }

        console.log("Still unconfirmed bundles: ", migrationBundleHashes);
      }
    }

    // Attach an event listener to keep track of the migration process
    addEventListener("MigrationProgress", callback)

    const manager = new AccountManager({
      storagePath: DB_STORAGE_PATH,
    })

    // Set Stronghold password
    manager.setStrongholdPassword(process.env.SH_PASSWORD)

    // IMPORTANT: SAVE THIS MNEMONIC SECURELY. IF YOU LOSE IT, YOU POTENTIALLY LOSE EVERYTHING.
    const mnemonic = manager.generateMnemonic()

    console.log("IMPORTANT: SAVE THIS MNEMONIC SECURELY. IF YOU LOSE IT, YOU POTENTIALLY LOSE EVERYTHING.", mnemonic);

    manager.storeMnemonic(SignerType.Stronghold, mnemonic)

    const account = await manager.createAccount({
      // Node url for the new network
      clientOptions: { node: CHRYSALIS_NODE, localPow: true, network: "chrysalis-mainnet" },
      alias: 'Migration',
    })

    console.log('Account created:', account.alias())
    // Nodes for the legacy network
    const nodes = LEGACY_NETWORK_NODES
    const seed = process.env.MIGRATION_SEED

    const migrationData = await manager.getMigrationData(
      nodes,
      seed,
      {
        // permanode for the legacy network
        permanode: LEGACY_PERMANODE,
        securityLevel: ADDRESS_SECURITY_LEVEL,
        // this is the default and from there it will check addresses for balance until 30 in a row have 0 balance
        // if not all balance got detected because a higher address index was used it needs to be increased here
        initialAddressIndex: 0
      }
    )

    console.log(migrationData)

    if (migrationData.balance > 0) {
      let input_batches = getMigrationBundles(migrationData.inputs)
      // create bundles with the inputs
      for (batch of input_batches) {
        try {
          const bundle = await manager.createMigrationBundle(seed, batch.inputs.map(input => input.index), {
            logFileName: 'iota-migration.log',
            // if the input is a spent address we do a bundle mining process which takes 10 minutes to reduce the amount 
            // of the parts of the private key which get revealed
            mine: batch.inputs[0].spent
          })
          migrationBundleHashes.push(bundle.bundleHash)
        } catch (e) {
          console.error(e);
        }
      }

      // Send all bundles to the Tangle and reattach them until they are confirmed
      for (bundleHash of migrationBundleHashes) {
        try {
          await manager.sendMigrationBundle(nodes, bundleHash)
        } catch (e) { console.error(e) }
      }
    } else {
      console.log('Detected 0 balance. Exiting.')

      process.exit(0)
    }
  } catch (e) {
    console.error(e);
  }
}

run()

const getMigrationBundles = (inputs) => {
  // Categorise spent vs unspent inputs
  const { spent, unspent } = inputs.reduce((acc, input) => {
    if (input.spent) {
      acc.spent.push(input)
    } else {
      acc.unspent.push(input)
    }
    return acc;
  }, { spent: [], unspent: [] })
  const unspentInputChunks = selectInputsForUnspentAddresses(unspent)
  const spentInputs = spent.filter((input) => input.balance >= MINIMUM_MIGRATION_BALANCE)
  return [
    ...spentInputs.map((input) => ({
      // Make sure for spent addresses, we only have one input per bundle    
      inputs: [input]
    })),
    ...unspentInputChunks.map((inputs) => ({ inputs }))
  ]
};

/**
 * Prepares inputs (as bundles) for unspent addresses.
 * Steps:
 *   - Categorises inputs in two groups 1) inputs with balance >= MINIMUM_MIGRATION_BALANCE 2) inputs with balance < MINIMUM_MIGRATION_BALANCE
 *   - Creates chunks of category 1 input addresses such that length of each chunk should not exceed MAX_INPUTS_PER_BUNDLE
 *   - For category 2: 
 *         - Sort the inputs in descending order based on balance;
 *         - Pick first N inputs (where N = MAX_INPUTS_PER_BUNDLE) and see if their accumulative balance >= MINIMUM_MIGRATION_BALANCE
 *         - If yes, then repeat the process for next N inputs. Otherwise, iterate on the remaining inputs and add it to a chunk that has space for more inputs
 *         - If there's no chunk with space left, then ignore these funds. NOTE THAT THESE FUNDS WILL ESSENTIALLY BE LOST!
 * 
 * NOTE: If the total sum of provided inputs are less than MINIMUM_MIGRATION_BALANCE, then this method will just return and empty array as those funds can't be migrated.
 * 
 * This method gives precedence to max inputs over funds. It ensures, a maximum a bundle could have is 30 inputs and their accumulative balance >= MINIMUM_MIGRATION_BALANCE
 * 
 * @method selectInputsForUnspentAddresses
 * 
 * @params {Input[]} inputs
 * 
 * @returns {Input[][]}
 */
const selectInputsForUnspentAddresses = (inputs) => {
  const totalInputsBalance = inputs.reduce((acc, input) => acc + input.balance, 0);

  // If the total sum of unspent addresses is less than MINIMUM MIGRATION BALANCE, just return an empty array as these funds cannot be migrated
  if (totalInputsBalance < MINIMUM_MIGRATION_BALANCE) {
    return [];
  }

  const { inputsWithEnoughBalance, inputsWithLowBalance } = inputs.reduce((acc, input) => {
    if (input.balance >= MINIMUM_MIGRATION_BALANCE) {
      acc.inputsWithEnoughBalance.push(input);
    } else {
      acc.inputsWithLowBalance.push(input);
    }

    return acc;
  }, { inputsWithEnoughBalance: [], inputsWithLowBalance: [] })

  let chunks = inputsWithEnoughBalance.reduce((acc, input, index) => {
    const chunkIndex = Math.floor(index / MAX_INPUTS_PER_BUNDLE)

    if (!acc[chunkIndex]) {
      acc[chunkIndex] = [] // start a new chunk
    }

    acc[chunkIndex].push(input)

    return acc
  }, [])

  const fill = (_inputs) => {
   _inputs.every((input) => {
      const chunkIndexWithSpaceForInput = chunks.findIndex((chunk) => chunk.length < MAX_INPUTS_PER_BUNDLE);

      if (chunkIndexWithSpaceForInput > -1) {
        chunks = chunks.map((chunk, idx) => {
          if (idx === chunkIndexWithSpaceForInput) {
            return [...chunk, input]
          }

          return chunk
        })

        return true;
      }

      // If there is no space, then exit
      return false;
    })
  }

  const totalBalanceOnInputsWithLowBalance = inputsWithLowBalance.reduce((acc, input) => acc + input.balance, 0)

  // If all the remaining input addresses have accumulative balance less than the minimum migration balance,
  // Then sort the inputs in descending order and try to pair the
  if (totalBalanceOnInputsWithLowBalance < MINIMUM_MIGRATION_BALANCE) {
    const sorted = inputsWithLowBalance.slice().sort((a, b) => b.balance - a.balance)

    fill(sorted)
  } else {
    let startIndex = 0

    const sorted = inputsWithLowBalance.slice().sort((a, b) => b.balance - a.balance)
    const max = Math.ceil(sorted.length / MAX_INPUTS_PER_BUNDLE);

    while (startIndex < max) {
      const inputsSubset = sorted.slice(startIndex * MAX_INPUTS_PER_BUNDLE, (startIndex + 1) * MAX_INPUTS_PER_BUNDLE)
      const balanceOnInputsSubset = inputsSubset.reduce((acc, input) => acc + input.balance, 0);

      if (balanceOnInputsSubset >= MINIMUM_MIGRATION_BALANCE) {
        chunks = [...chunks, inputsSubset]
      } else {
        fill(inputsSubset)
      }

      startIndex++;
    }
  }

  return chunks;
};
```
