# Examples

## Account manager and individual accounts
First of all, let's initialize (open) a secure storage for individual accounts (backed up by Stronghold by default) using `AccountManager` instance:

```python
import iota_wallet as iw

account_manager = iw.AccountManager(
    storage='Stronghold', storage_path='./alice-database'
) #note: `storage` and `storage_path` have to be declared together

account_manager.set_stronghold_password("password")

# mnemonic should be set only for new storage
# once the storage has been initialized earlier then you can ommit this step
account_manager.store_mnemonic("Stronghold")
```
* Storage is initialized under the given path (`./alice-database`)
* Password is set (`password`)
* Only during the initialization new database: Stronghold mnemonic is automatically generated and stored. Note: the library can also generate mnemonic for other signer types, such as `LedgerNano`, or `LedgerNanoSimulator`.
* Alternatively, own mnemonic can be stored using `store_mnemonic(signer_type, mnemonic=None)` method and validated via `verify_mnemonic(mnemonic): None` method later

Just a reminder. Any information stored via Stronghold is encrypted at rest.

The `AccountManager` instance is the main vehicle while dealing with the underlying wallet storage. 

### Accounts
The library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically.

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

print('Account created...')
```
Once an account has been created you get an instance of it using the following methods: `get_account(account_id: str)` or `get_accounts()`.
An account can be then referred to via `index`, `alias` or one of its generated `addresses`.

```python
account = account_manager.get_account("Alice")
```

Several api calls can be performed via `account` instance. Note: it is a good practice to sync the given account with the Tangle every time you work with `account` instance to rely on the latest information available: `account.sync().execute()`.

The most common methods:
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
Take a closer look at the output above and check the beginning of both addresses. As mentioned in [overview chapter](./welcome.md) there are two human-readable identifiers in IOTA 1.5 network: `iota` (mainnet) and `atoi` (testnet).

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
In the detailed view per individual addresses are visible also the individual transactions (also known as `wallet messages`). The amount can be also double checked using [Tangle Explorer](https://explorer.iota.org/chrysalis/addr/atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg). 

