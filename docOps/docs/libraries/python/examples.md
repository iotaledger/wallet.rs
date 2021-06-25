# Examples

This section will guide you through several examples using the python binding of the `wallet.rs` library. You can also find the code for the examples in the `/bindings/python/examples` folder in the [official GitHub repository](https://github.com/iotaledger/wallet.rs/tree/develop/bindings/python/examples).

All the examples in this section expect you to set your custom password  in the _.env_ file:

```bash
SH_PASSWORD="here is your super secure password"
```

## Account Manager and Individual Accounts

You can initialize (open) a secure storage for individual accounts.  The storage is backed up by `Stronghold` by default, using an AccountManager instance.  

The following example creates a new database and account:
```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv


# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

account_manager = iw.AccountManager(
    storage_path='./alice-database'
)  # note: `storage` and `storage_path` have to be declared together

account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# mnemonic (seed) should be set only for new storage
# once the storage has been initialized earlier then you should omit this step
account_manager.store_mnemonic("Stronghold")
```
* Storage is initialized under the given path (`./alice-database`)
* The password is set based on your password in _.env_ file ( `manager.setStrongholdPassword(process.env.SH_PASSWORD)` )
* When you initialize the new database, a Stronghold mnemonic (seed) is automatically generated and stored by default ( `manager.storeMnemonic(SignerType.Stronghold)` ).
* The seed should be set only for the first time. In order to open already initialized database, you can simply use your password.

The storage is encrypted at rest, so you need a strong password and location where to place your storage. 

:::warning
We highly recommended that you to store your `Stronghold` password encrypted on rest and separated from `Stronghold` snapshots. 

Deal with the password with utmost care.
:::

 The storage comprises two things:
* A single file called _wallet.stronghold_ , which contains _seed_ and is secured by `Stronghold` and encrypted at rest. The generated seed (mnemonic) serves as a cryptographic key from which all accounts and related addresses are generated.
* Other data used by library that is stored under the _db_ sub-directory.  The includes account information, generated addresses, fetched messages, etc.  This data is used to speed up some operations, such as account creation, address generation, etc.

One of the key principles behind `Stronghold` based storage is that no one can extract a seed from the storage. You deal with all accounts purely via an _AccountManager_ instance.  All complexities are hidden under the hood and dealt with securely.

If you also want to store a seed somewhere else, you can use the `AccountManager.generateMnemonic()` method. This method will generate a random seed, and it can be used before the actual account initialization.

You can find detailed information about seed generation at [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#seed).

### Accounts

The `wallet.rs` library uses a model of individual accounts to separate individual users/clients from each other. It is possible to generate multiple addresses for each account deterministically. 

Each account is related to a specific IOTA network (mainnet / testnet), which is referenced by node properties such as node url.  In this example, the `Chrysalis` testnet balancer.

For more information about _client_options_ , please refer to [Wallet Python API Reference](api_reference.md#clientoptions).

```python
# ... continue from prev example 1a

# general Tangle specific options
client_options = {
    "nodes": [
        {
            "url": "https://api.hornet-0.testnet.chrysalis2.com",
            "auth": None,
            "disabled": False
        }
    ],
    "local_pow": True
}

# an account is generated with the given alias via `account_initialiser`
account_initialiser = account_manager.create_account(client_options)
account_initialiser.alias('Alice')

# initialise account based via `account_initialiser`
# store it to db and sync with Tangle
account = account_initialiser.initialise()
print(f'Account created: {account.alias()}')
```
_Alias_ should be unique, and it can be any string that you see fit. The _alias_ is usually used to identify the account later on. Each account is also represented by an _index_ which is incremented by 1 every time new account is created. 
Any account can be then referred to by its _index_ , _alias_ or one of its generated _addresses_ .

Once an account has been created, you retrieve an instance of it using the following methods: 
- `get_account(account_id: str)`
- `get_accounts()` .


You can get an overview of all available accounts by running the following snippet:
```python
for acc in account_manager.get_accounts():
  print(f"Account alias: {acc.alias()}; network: {acc.bech32_hrp()}")
```

You can get and instance of a specific account using the `account_manager.get_account("ALIAS")`, replacing _"ALIAS"_ for the given alias:
```python
account = account_manager.get_account("Alice")
```

Several API calls can be performed via an _account_ instance.

:::info
It is a good practice to sync the given _account_ with the Tangle every time you work with an _account_ instance to retrieve the latest information available.  You can do this using the `account.sync()` method.  By default, `account.sync()` is performed automatically on `send` , `retry` , `reattach` and `promote` API calls.
:::

The most common methods of _account_ instance are:
* `account.alias()` : returns an alias of the given account.
* `account.addresses()` : returns list of addresses related to the account.
* `account.get_unused_address()` : returns a first unused address.
* `account.is_latest_address_unused()` : queries the Tangle and returns a _bool_ whether latest address was already used.
* `account.generate_address()` : generates a new address for the address index incremented by 1.
* `account.balance()` : returns the balance for the given account.
* `account.sync()` : syncs the account information with the tangle.

## Generating Address(es)

Each _account_ can have multiple _addresses_ . _Addresses_ are generated deterministically based on the _account_ and _address_ index. This means that the combination of _account_ and index uniquely identifies the given address.

There are two types of addresses, _internal_ and _public_ (external), and each set of addresses is independent of each other and has independent _index_ id.

* _Public_ addresses are created by `account.generateAddress()` and are indicated as `internal=false` (public)
* _Internal_ addresses are also called _change_ addresses. _Internal_ addresses are used to store the excess funds and are indicated as `internal=true`.

This approach is also known as a *BIP32 Hierarchical Deterministic wallet (HD Wallet)*.

:::info
The IOTA 1.5 (Chrysalis) network supports reusing addresses multiple times.
::: 

You can use the following example to generate a new address via an instance of _account_ which was retrieved using an _account_manager_ instance:

```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example generates a new address.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)

account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# get a specific instance of some account
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# generate new address
address = account.generate_address()
print(f'New address: {address}')

# print all addresses generated so far
print("List of addresses:")
print(account.addresses())

# You can also get the latest unused address
last_address_obj = account.latest_address()
print(f"Last address: {last_address_obj['address']}")
```

Example output:
```json
[{
        'address': {
            'inner': 'atoi1qzy79ew8x4hn4dsr0t3j8ce8hdwdrh8xzx85x2gkse6k0fx2jkyaqdgd2rn'
        },
        'balance': 0,
        'key_index': 0,
        'internal': False,
        'outputs': []
    },
    {
        'address': {
            'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'
        },
        'balance': 0,
        'key_index': 1,
        'internal': False,
        'outputs': []
    }
]
```
There are two human-readable prefixes in IOTA 1.5 network: _iota_ (mainnet) and _atoi_ (testnet). If you take a close look at the addresses in the output, you will be able to notice that both of them start with _atoi_ , and are therefore testnet addresses. 

You can find detailed information about generating addresses at the [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#addresskey-space).

## Checking the Balance

Before we continue further, please visit the [IOTA testnet faucet service](https://faucet.testnet.chrysalis2.com/) and send some tokens to your testnet addresses.

![IOTA Faucet Service](../../../static/img/libraries/screenshot_faucet.png)

You can use the following example to sync your accounts and retrieve their balances.


```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example checks the account balance.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)

account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# get a specific instance of some account
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# get total balance for the account
print("Total balance:")
print(account.balance())

print("Balance per individual addresses:")
print(account.addresses())
```

Example output:
```json
Total balance:
{
    'total': 10000000,
    'available': 10000000,
    'incoming': 10000000,
    'outgoing': 0
}
        
Balance per individual addresses:
[{
        'address': {
            'inner': 'atoi1qzy79ew8x4hn4dsr0t3j8ce8hdwdrh8xzx85x2gkse6k0fx2jkyaqdgd2rn'
        },
        'balance': 0,
        'key_index': 0,
        'internal': False,
        'outputs': []
    },
    {
        'address': {
            'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'
        },
        'balance': 10000000,
        'key_index': 1,
        'internal': False,
        'outputs': [{
            'transaction_id': '1c88c91fe0a8eed074b5ccdfdad52403d7908d157b231ae1ef28b0e20ba14e8e',
            'message_id': 'f1575f984f7fda6e9b3e23e96ef3304fcd0ba4ce323af3920856a427fabe1abe',
            'index': 0,
            'amount': 10000000,
            'is_spent': False,
            'address': {
                'inner': 'atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg'
            }
        }]
    },
    {
        'address': {
            'inner': 'atoi1qpvnsgygzal4vkxhlc0ew7c6c6csnjr72x5rgn3txqswrsa2xfrec8v04f7'
        },
        'balance': 0,
        'key_index': 2,
        'internal': False,
        'outputs': []
    }
]
```
In the detailed view per individual addresses, there is also _outputs_ section.  The _outputs_ shows all the transactions (also known as _wallet message(s)_ ), which are related to that _address_, and therefore account for the balance. 

You can also check the balance using the [Tangle Explorer](https://explorer.iota.org/testnet/addr/atoi1qzht4m2jt0q50lhlqa786pcx6vardm4xj8za72fezde6tj39acatq5zh2cg).

:::info
IOTA is based on _Unspent Transaction Output_ model. You can find a detailed explanation in the [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#unspent-transaction-output-utxo).
:::

## Sending Tokens

The process of sending tokens via _wallet.rs_ can be described as follows:
1. Create instance of `iota_wallet.Transfer()` class with the following mandatory arguments: _amount_, _address_ and _remainder_value_strategy_ . 
The _remainder_value_strategy_ argument can be either: 
   - `ReuseAddress` 
   - `ChangeAddress` 
2. Once you have created an instance of `iota_wallet.Transfer()` , you can send the tokens using the `transfer()` function of the _Account_ instance.

:::info
We highly recommend that you sync the account information with the Tangle by running the `account.sync().execute()` method before doing anything with the account. This way you can ensure that you rely on the latest available information.
:::

```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example sends IOTA toens to an address.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

print("Selecting a specific account")
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()} selected')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

print(f"Available balance {account.balance()['available']}")

# TODO: Replace with the address of your choice!
transfer = iw.Transfer(
    amount=1_000_000,
    address='atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r',
    remainder_value_strategy='ReuseAddress'
)

# Propogate the Transfer to Tangle
# and get a response from the Tangle
node_response = account.transfer(transfer)
print(
    node_response
)
```

The previous snippet should have a similar output to the following JSON object:
```json
{
    'id': '9d3c401d59b0a87f6fbaa58582bb71e1858d63336421ccbae834821d9be113d3',
    'version': 1,
    'parents': ['66009ff08637c3e74340fb9e09e30e3c4453728c857fd425df2d2e0587af6426',
        '6da392ac35f73594bf5509fb5c3304e972b36313ce98f2cc63def7cde2054b53',
        '9157b29cbffcd5c9669cf22004fbc557354e5ade7268f5bfe25fbc75ab29e3b1',
        'bfe860e09350cd3b8db90611e78e03fdda654139a4b34e68e4b1bb07528b2bef'
    ],
    'payload_length': 233,
    'payload': {
        'transaction': [{
            'essence': {
                'regular': {
                    'inputs': [{
                        'transaction_id': '692d6660084dd3b6341ef4f761bc8b8bb27ac35bb0b352bfb030f2c80753815b',
                        'index': 0,
                        'metadata': {
                            'transaction_id': '692d6660084dd3b6341ef4f761bc8b8bb27ac35bb0b352bfb030f2c80753815b',
                            'message_id': 'c6284e0cc2a6383474782d4e6b6cfaf16c1831c8875cca262982782758a248c0',
                            'index': 0,
                            'amount': 10000000,
                            'is_spent': False,
                            'address': {
                                'inner': 'atoi1qq24vlx53qdskyfw6940xa2vg55ma5egzyqv6glq23udx3e0zkmmg97cwze'
                            }
                        }
                    }],
                    'outputs': [{
                            'address': 'atoi1qq24vlx53qdskyfw6940xa2vg55ma5egzyqv6glq23udx3e0zkmmg97cwze',
                            'amount': 9000000
                        },
                        {
                            'address': 'atoi1qpvnsgygzal4vkxhlc0ew7c6c6csnjr72x5rgn3txqswrsa2xfrec8v04f7',
                            'amount': 1000000
                        }
                    ],
                    'payload': None
                }
            },
            'unlock_blocks': [{
                'signature': {
                    'public_key': [15... < TRIMMED > ...],
                    'signature': [210... < TRIMMED > ...]
                },
                'reference': None
            }]
        }],
        'milestone': None,
        'indexation': None
    },
    'timestamp': 1615132552,
    'nonce': 274654,
    'confirmed': None,
    'broadcasted': True,
    'incoming': False,
    'value': 1000000,
    'remainder_value': 9000000
}}
```
This is a _wallet message_ that fully describes the given transaction.

To understand all aspects of messages, you will need to get familiar with concept of _UTXO_ . You can find detailed information in the [UTXO section in the Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#unspent-transaction-output-utxo).

You can double-check the message using [Tangle Explorer](https://explorer.iota.org/) using its _node_response['id']_.  Please make sure you select the right network.

If you have used the _ChangeAddress remainder_value_strategy_, the message will transfer tokens to the target address as well as new _internal_ address within the given account (`internal=True`). 

You can find detailed information about messages and payloads in the [Developer Guide to Chrysalis](https://chrysalis.docs.iota.org/guides/dev_guide.html#messages-payloads-and-transactions).

### Reattachments

If you need to reattach a message, you should use the [`iota_wallet.promote(account_id, message_id)`](api_reference.md#promoteaccount_id-message_id-walletmessagewalletmessage) or [`iota_wallet.reattach(account_id, message_id)`](api_reference.md#reattachmessage_id-walletmessagewalletmessage) methods, sending your _account_id_ and _message_id_ as arguments.

### List of Messages (transactions)

You can query for a list of all particular messages (transactions) related to the given account using [ `account.list_messages()` ](api_reference.md#list_messagescount-from-message_type-optional-listwalletmessagewalletmessage) method, and the related [ `account.message_count()` ](api_reference.md#message_countmessage_type-optional-int) method.

You can use those methods to check whether a message is confirmed, broadcast, etc. You should always _Sync_ the account with the Tangle before checking confirmation status.

You can use the following example to _sync_ an _account_ , and list all the messages related to the _account_ . 
```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example sends IOTA toens to an address.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

account = account_manager.get_account('Alice')
print(f'Account: {account.alias()} selected')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

for ac in account.list_messages():
    print(f"message {ac['id']}; confirmation status = {ac['confirmed']}'")
```

### Dust Protection

The network uses a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide.html#dust-protection) protocol to prevent malicious actors from spamming the network while also keeping track of the unspent amount ( _UTXO_ ).

:::info
“... micro-transaction below 1Mi of IOTA tokens can be sent to another address if there is already at least 1Mi on that address. 
That's why we sent 1Mi in the last example to comply with the protection.”
:::

Dust protection also means you can't leave less than 1Mi on a spent address (leave a dust behind).

## Backup Database

Due to security practices that are incorporated in the `Stronghold's` DNA, there's no way to retrieve a seed, as seeds are encrypted at rest.  Therefore, if you're using the default options, backing up the seed storage is a very important task. 

The following example will guide you in backing up your data in secure files. You can move this file to another app or device, and restore it.

```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example backups your data in a secure file.
# You can move this file to another app or device and restore it.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

backup_dir_path = './backup'
if not os.path.exists(backup_dir_path):
    os.makedirs(backup_dir_path)
backup_file_path = account_manager.backup(backup_dir_path, STRONGHOLD_PASSWORD)

print(f'Backup path: {backup_file_path}')
```

Output:
```plaintext
Backup path: ./backup/2021-03-07T18-24-06-iota-wallet-backup-wallet.stronghold
```
Alternatively, you can create a copy of the _wallet.stronghold_ file and use it as seed backup. This can be achieved by a daily [_cronjob_](https://linux.die.net/man/1/crontab), [_rsync_](https://linux.die.net/man/1/rsync) or [_scp_](https://linux.die.net/man/1/scp) with a datetime suffix for example.

## Restore a Database

To restore a database via `wallet.rs`, you will need to:
1. Create new empty database with a password (without mnemonic seed)
2. Import all accounts from the file that has been backed up earlier

The following example restores a secured backup file:

```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example restores a secured backup file.
account_manager = iw.AccountManager(
    storage_path='./alice-database-restored'
)

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

#  Add the path to the file from example 5-backup.js
#  for example: ./backup/2021-03-04T15-31-04-iota-wallet-backup-wallet.stronghold
backup_file_path = r'./backup/2021-03-31T14-45-23-iota-wallet-backup-wallet.stronghold'

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.import_accounts(backup_file_path, STRONGHOLD_PASSWORD)
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')
```

Since the backup file is just a copy of the original database it can be also be renamed to _wallet.stronghold_ and opened in a standard way.

```python
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")
```

## Listening to Events

`wallet.rs` library is able to listen to several supported event. As soon as the event occurs, a provided callback will be triggered.

You can add any of the following event listeners:
* `on_balance_change(callback): id`
* `on_new_transaction(callback): id`
* `on_confirmation_state_change(callback): id`
* `on_reattachment(callback): id`
* `on_broadcast(callback): id`
* `on_error(callback): id`
* `on_stronghold_status_change(callback): id`

Once you have registered an event listener using, the function will return an _id_ for the listener as a list[Bytes].  
You can later use this _id_ to remove a listener by using the corresponding method described below:

* `remove_balance_change_listener(id)` 
* `remove_new_transaction_listener(id)` 
* `remove_confirmation_state_change_listener(id)` 
* `remove_reattachment_listener(id)` 
* `remove_broadcast_listener(id)` 
* `remove_error_listener(id)` 
* `remove_stronghold_status_change_listener(id)` 

The following example set's up a listener for the _on_balance_change_ event using an event-based pattern:
```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

import threading
import time
import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

result_available = threading.Event()


def balance_changed_event_processing(event):
    print(f'On balanced changed: {event}')
    result_available.set()


# This example shows some events.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# Get the latest unused address
last_address_obj = account.latest_address()
print(f"Address: {last_address_obj['address']}")

# Use the Chrysalis Faucet to send testnet tokens to your address:
print('Fill your address with the Faucet: https://faucet.tanglekit.de/')

iw.on_balance_change(balance_changed_event_processing)
print("Waiting for external event (on_balance_changed)...")

# wait for results to be available before continue
# will not wait longer than 360 seconds
result_available.wait(timeout=360)

print("Done.")
```

Expected output:
```plaintext
Account: Alice
Syncing...
Address: {'inner': 'atoi1qquszp0hzfsrgx4vx58dfg4v6eh20d2k3ddfgg9dt5778c2egc9uyw7g457'}
Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/
Waiting for external event (on_balance_changed)...
On balanced changed: {"indexationId":"c3a7a1ab8ba78460954223a704693d088ddd0388681ac6cc1dd964a388d1a619","accountId":"wallet-account://e51a6285ea2d8cbdf5b6da2b85a8344f619d798d869ef4fb88c5fac0e653d6cc","address":"atoi1qquszp0hzfsrgx4vx58dfg4v6eh20d2k3ddfgg9dt5778c2egc9uyw7g457","balanceChange":{"spent":0,"received":10000000}}
Done.
```

Alternatively, events can be consumed via queue-base pattern as shown in the following example:

```python
# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet
import threading
import queue
import time
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the Stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example shows how to listen to on_balance_change event.

# The queue to store received events
q = queue.Queue()


def worker():
    """The worker to process the queued events.
    """
    while True:
        item = q.get(True)
        print(f'Get event: {item}')
        q.task_done()


def balance_changed_event_processing(event):
    """Processing function when event is received.
    """
    print(f'On balanced changed: {event}')
    q.put(event)


# Get the acount manager
manager = iota_wallet.AccountManager(
    storage_path='./alice-database')

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# Get the account
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# Get the latest unused address
last_address_obj = account.latest_address()
print(f"Address: {last_address_obj['address']}")

# turn-on the worker thread
threading.Thread(target=worker, daemon=True).start()

# listen to the on_balance_change event
iota_wallet.on_balance_change(balance_changed_event_processing)

# Use the Chrysalis Faucet to send testnet tokens to your address:
print(
    f"Fill your Address ({last_address_obj['address']['inner']}) with the Faucet: https://faucet.tanglekit.de/")
print("To see how the on_balance_change is called, please send tokens to the address in 1 min")
time.sleep(60)

# block until all tasks are done
q.join()
print('All work completed')
```
