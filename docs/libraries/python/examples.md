# Examples

## Account manager and individual accounts
First of all, let's initialize (open) a secure storage for individual accounts (backed up by Stronghold by default) using `AccountManager` instance:

```python
import iota_wallet as iw

account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
) #note: `storage` and `storage_path` have to be declared together

account_manager.set_stronghold_password("password")

# mnemonic (seed) should be set only for new storage
# once the storage has been initialized earlier then you should omit this step
account_manager.store_mnemonic("Stronghold")
```
* Storage is initialized under the given path (`./alice-database`)
* Password is set (`password`)
* Only during the initialization new database: Stronghold mnemonic (seed) is automatically generated and stored. Note: the library can also generate mnemonic for other signer types, such as `LedgerNano`, or `LedgerNanoSimulator`.
* Alternatively, own mnemonic can be stored using `store_mnemonic(signer_type, mnemonic=None)` method and validated via `verify_mnemonic(mnemonic): None` method later

Just a reminder. Any information stored via Stronghold is encrypted at rest.

The `AccountManager` instance is the main vehicle while dealing with the underlying wallet storage. 

### Accounts
The library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically. Please note, it is important to declare on which IOTA network should be the given account created (argument `node`).

```python
# general Tangle specific options
client_options = {
    'node': 'https://api.lb-0.testnet.chrysalis2.com',
    'local_pow': True,
}

# an account is generated with the given alias via `account_initialiser`
account_initialiser = account_manager.create_account(client_options)
account_initialiser.alias('Alice')

# initialise account based via `account_initialiser`
# store it to db and sync with Tangle
account = account_initialiser.initialise()

print(f'Account `{account.alias()}` created...')
```
Once an account has been created you get an instance of it using the following methods: `get_account(account_id: str)` or `get_accounts()`.
An account can be then referred to via `index`, `alias` or one of its generated `addresses`. The network against which the account is active can be checked via `account.bech32_hrp()`.

Overview of all accounts:
```python
for acc in account_manager.get_accounts():
  print(f"Account alias: {acc.alias()}; network: {acc.bech32_hrp()}")
```

Get the instance of a specific account:
```python
account = account_manager.get_account("Alice")
```

Several api calls can be performed via `account` instance. Note: it is a good practice to sync the given account with the Tangle every time you work with `account` instance to rely on the latest information available: `account.sync().execute()`.

The most common methods:
* `account.alias()`: returns an alias of the given account
* `account.addresses()`: returns list of addresses related to the account
* `account.get_unused_address()`: returns a first unused address
* `account.is_latest_address_unused()`: it queries the Tangle and returns `bool` whether latest address was already used
* `account.generate_address()`: generate a new address for the address index incremented by 1
* `account.balance()`: returns the balance for the given account
* `account.sync()`: sync the account information with the tangle

## Generating address(es)
Each account can posses multiple addresses. Addresses are generated deterministically based on the account and address index. It means that the combination of account and index uniquely identifies the given address.

_Note: You may remember IOTA 1.0 network in which addresses were not reusable. It is no longer true and addresses can be reused multiple times in IOTA 1.5 (Chrysalis) network._

Addresses are generated via instance of `account` that is gotten from the `account_manager` instance:

```python
import iota_wallet as iw

account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

# get a specific instance of some account
account = account_manager.get_account("Alice")

# sync the underlying storage with the Tangle
print("Syncing with the Tangle...")
account.sync().execute()

# generate new address
account.generate_address()

# print all addresses generated so far
print("List of addresses:")
print(account.addresses())

# sync the underlying storage with the Tangle
# new address will become "known" to Tangle
print("Syncing with the Tangle...")
account.sync().execute()
```

```json
[{'address': {'inner': 'atoi1qzy79ew8x4hn4dsr0t3j8ce8hdwdrh8xzx85x2gkse6k0fx2jkyaqdgd2rn'},
  'balance': 0,
  'key_index': 0,
  'internal': False,
  'outputs': []},
 {'address': {'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'},
  'balance': 0,
  'key_index': 1,
  'internal': False,
  'outputs': []}]
```
Take a closer look at the output above and check the beginning of both addresses. As mentioned in [overview chapter](./welcome.md) there are two human-readable prefixes in IOTA 1.5 network: `iota` (mainnet) and `atoi` (testnet).

## Checking the balance
Before we continue further, go to [IOTA testnet faucet service](https://faucet.testnet.chrysalis2.com/) and send to your testnet addresses some tokens:
![faucet screenshot](screenshot_faucet.png)


```python
import iota_wallet as iw

account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

# get a specific instance of some account
account = account_manager.get_account("Alice")

# sync the underlying storage with the Tangle
print("Syncing with the Tangle...")
account.sync().execute()

# get total balance for the account
print("Total balance:")
print(account.balance())

print("Balance per individual addresses:")
print(account.addresses())
```

```json
Total balance:
{'total': 10000000, 'available': 10000000, 'incoming': 10000000, 'outgoing': 0}

Balance per individual addresses:
[{'address': {'inner': 'atoi1qzy79ew8x4hn4dsr0t3j8ce8hdwdrh8xzx85x2gkse6k0fx2jkyaqdgd2rn'},
  'balance': 0,
  'key_index': 0,
  'internal': False,
  'outputs': []},
 {'address': {'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'},
  'balance': 10000000,
  'key_index': 1,
  'internal': False,
  'outputs': [{'transaction_id': '1c88c91fe0a8eed074b5ccdfdad52403d7908d157b231ae1ef28b0e20ba14e8e',
    'message_id': 'f1575f984f7fda6e9b3e23e96ef3304fcd0ba4ce323af3920856a427fabe1abe',
    'index': 0,
    'amount': 10000000,
    'is_spent': False,
    'address': {'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'}}]},
 {'address': {'inner': 'atoi1qpvnsgygzal4vkxhlc0ew7c6c6csnjr72x5rgn3txqswrsa2xfrec8v04f7'},
  'balance': 0,
  'key_index': 2,
  'internal': False,
  'outputs': []}]
```
In the detailed view per individual addresses, there is also `outputs` section that indicates the latest transaction (also known as `wallet message`) "responsible" for the current amount. The amount can be also double checked using [Tangle Explorer](https://explorer.iota.org/chrysalis/addr/atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg).

## Sending tokens
The process of sending tokens via `wallet.rs` can be described as follows:
* Create instance of `iota_wallet.Transfer()` class with the following mandatory arguments: `amount`, `address`, `bench32_hrp` and `remainder_value_strategy`
* `bench32_hrp` indicates which network should be used to send tokens thru, and can be gotten from `account.bech32_hrp()` function
* `remainder_value_strategy` can be: `ReuseAddress` or `ChangeAddress`. You may be familiar with a concept `changing address with every spent` in IOTA 1.0. It is not an issue in IOTA 1.5 world but it may still become handy depending on your use case
* once instance of `iota_wallet.Transfer()` is created, it can be sent via `transfer()` function of the `Account` instance
* Needless to repeat, always sync the account information with the Tangle before do anything with the account

```python
import iota_wallet as iw

account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

print("Selecting a specific account")
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()} selected')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

print(f"Available balance {account.balance()['available']}")

# TODO: Replace with the address of your choice!
target_address = "atoi1qpvnsgygzal4vkxhlc0ew7c6c6csnjr72x5rgn3txqswrsa2xfrec8v04f7"
transfer = iw.Transfer(
    amount=1_000_000,
    address=target_address,
    bench32_hrp=account.bech32_hrp(),
    remainder_value_strategy="ReuseAddress"
)

# Propogate the Transfer to Tangle
# and get a response from the Tangle
node_response = account.transfer(transfer)
print(
    node_response
)
```

You receive an output similar to the following one:
```json
{'id': '9d3c401d59b0a87f6fbaa58582bb71e1858d63336421ccbae834821d9be113d3',
 'version': 1,
 'parents': ['66009ff08637c3e74340fb9e09e30e3c4453728c857fd425df2d2e0587af6426',
  '6da392ac35f73594bf5509fb5c3304e972b36313ce98f2cc63def7cde2054b53',
  '9157b29cbffcd5c9669cf22004fbc557354e5ade7268f5bfe25fbc75ab29e3b1',
  'bfe860e09350cd3b8db90611e78e03fdda654139a4b34e68e4b1bb07528b2bef'],
 'payload_length': 233,
 'payload': {'transaction': [{'essence': {'regular': {'inputs': [{'transaction_id': '692d6660084dd3b6341ef4f761bc8b8bb27ac35bb0b352bfb030f2c80753815b',
        'index': 0,
        'metadata': {'transaction_id': '692d6660084dd3b6341ef4f761bc8b8bb27ac35bb0b352bfb030f2c80753815b',
         'message_id': 'c6284e0cc2a6383474782d4e6b6cfaf16c1831c8875cca262982782758a248c0',
         'index': 0,
         'amount': 10000000,
         'is_spent': False,
         'address': {'inner': 'atoi1qq24vlx53qdskyfw6940xa2vg55ma5egzyqv6glq23udx3e0zkmmg97cwze'}}}],
      'outputs': [{'address': 'atoi1qq24vlx53qdskyfw6940xa2vg55ma5egzyqv6glq23udx3e0zkmmg97cwze',
        'amount': 9000000},
       {'address': 'atoi1qpvnsgygzal4vkxhlc0ew7c6c6csnjr72x5rgn3txqswrsa2xfrec8v04f7',
        'amount': 1000000}],
      'payload': None}},
    'unlock_blocks': [{'signature': {'public_key': [15...<TRIMMED>...],
       'signature': [210...<TRIMMED>...]},
      'reference': None}]}],
  'milestone': None,
  'indexation': None},
 'timestamp': 1615132552,
 'nonce': 274654,
 'confirmed': None,
 'broadcasted': True,
 'incoming': False,
 'value': 1000000,
 'remainder_value': 9000000}
```
It is a `wallet message` that fully describes the given transaction. Please, kindly get yourself familiar with a concept of [UTXO](https://chrysalis.docs.iota.org/introduction/what_is_chrysalis.html#switch-to-utxo-model) to understand all aspects of messages.

The given message can be double checked via Tangle Explorer using `node_response['ID']` field ([Tangle Explorer](https://explorer.iota.org/chrysalis/message/9d3c401d59b0a87f6fbaa58582bb71e1858d63336421ccbae834821d9be113d3)).

Needless to say, if `remainder_value_strategy` == `ChangeAddress` is used, the given message transfer tokens to target address as well as new address within the given account (`internal`). 

### Reattachments
If message reattachment is needed then `account_id` and `message_id` is passed to `iota_wallet.promote(account_id, message_id)` or `iota_wallet.reattach(account_id, message_id)`.

### List of messages (transactions)
List of all particular messages (transactions) related to the given account get be obtained via: `account.list_messages()` and related `account.message_count()`. Those can be used also to check whether the given message was confirmed/broadcasted, etc. Needless to say, sync the account with the Tangle before checking confirmation status:
```python
account = account_manager.get_account('Alice')
account.sync().execute()
for ac in account.list_messages():
    print(f"message {ac['id']}; confirmation status = {ac['confirmed']}'")
```

### Dust protection
Please note, there is also implemented a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide.html#dust-protection) mechanism in the network protocol to avoid malicious actors to spam network in order to decrease node performance while keeping track of unspent amount (`UTXO`):
> "... microtransaction below 1Mi of IOTA tokens [can be sent] to another address if you already have at least 1Mi on that address"
That's why we did send 1Mi in the given example to comply this protection.

## Backup database
Underlying database (`Stronghold` by default) is encrypted at rest and there is no way how to get a seed from it due to security practices that are incorporated in the Stronghold DNA. It means you are dealing with the database as an atomic unit that includes all wallet information.

So backing up the database is very important task from this respect.
```python
import iota_wallet as iw
import os

# This example backups your data in a secure file.
# You can move this file to another app or device and restore it.
account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

backup_dir_path = './backup'
if not os.path.exists(backup_dir_path):
    os.makedirs(backup_dir_path)
backup_file_path = account_manager.backup(backup_dir_path)
print(f'Backup path: {backup_file_path}')
```

```plaintext
Backup path: ./backup/2021-03-07T18-24-06-iota-wallet-backup-wallet.stronghold
```

## Restore database
The process of restoring underlying database via `wallet.rs` can be described as follows:
* create new empty database with a password (without mnemonic [seed])
* import all accounts from the file that has been backed up earlier
```python
import iota_wallet as iw

# This example restores a secured backup file.
account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database-restored'
)
account_manager.set_stronghold_password("password")

#  Add the path to the file from example 5-backup.js
#  for example: ./backup/2021-03-07T18-24-06-iota-wallet-backup-wallet.stronghold
backup_file_path = r'./backup/2021-03-07T18-24-06-iota-wallet-backup-wallet.stronghold'

account_manager.import_accounts(backup_file_path, 'password')
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')
```

Since the backup file is just a copy of the original database it can be alternatively also renamed to `wallet.stronghold` and opened in a standard way:
```python
account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")
```
