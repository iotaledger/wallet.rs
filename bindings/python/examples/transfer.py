# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os


# Get the stronghold password, please get the password from your environment
# NEVER assign your password directly in the code!
STRONGHOLD_PASSWORD = "UNSAFEPASSWORD"

# note: `storage` and `storage_path` have to be declared together
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)

account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# mnemonic (seed) should be set only for new storage
# once the storage has been initialized earlier then you should omit this step
account_manager.store_mnemonic("Stronghold")

# General Tangle specific options
client_options = {
    "primary_node": None,
    "primary_pow_node": None,
    "nodes": [
        {
            "url": "https://api.lb-0.h.chrysalis-devnet.iota.cafe",
            "auth": None,
            "disabled": False
        }
    ],
    "local_pow": None,
    "request_timeout": 60000,
    "api_timeout": {
        "PostMessage": 3000
    }
}

# An account is generated with the given alias via `account_initializer`
account_initializer = account_manager.create_account(client_options)
account_initializer.alias('Alice')

# Initialise account based via `initializer`
# Store it to db and sync with Tangle
account = account_initializer.initialise()
print(f'Account created: {account.alias()}')


# generate new address
address = account.generate_address()
print(f'New address: {address}')

# print all addresses generated so far
print("List of addresses:")
print(account.addresses())

# You can also get the latest unused address
last_address_obj = account.latest_address()
print(f"Last address: {last_address_obj['address']}")


# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# get total balance for the account
print("Total balance:")
print(account.balance())

print("Balance per individual addresses:")
print(account.addresses())


print('Websites to send tokens: https://faucet.chrysalis-devnet.iota.cafe/')
first_address = account.addresses()[0]['address']['inner']
print(f'Send tokens to {first_address}')
print(
    f'After sending, check https://explorer.iota.org/devnet/addr/{first_address}')
print('Please press enter to continue...')
input()

print(f"Available balance {account.balance()['available']}")

# transfer with a single output
# NOTE: Replace with the address of your choice!
transfer = iw.Transfer(
    amount=1_000_000,
    address='atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r',
    remainder_value_strategy='ReuseAddress'
)

node_response = account.transfer(transfer)
message_id = node_response['id']
print(f'Please check https://explorer.iota.org/devnet/addr/{message_id}')

# transfer with multiple outputs
# NOTE: Replace with the addresses of your choice!
transfer_outputs = [
    {"address": 'atoi1qrzlf0weq5x72qypl8falyw9fgwp6hfqu5kzwkf8aujjqxj36f3dx99rvqu', "amount": 1_000_000},
    {"address": 'atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r', "amount": 1_000_000}
]

transfer = iw.TransferWithOutputs(outputs=transfer_outputs, remainder_value_strategy="ReuseAddress")

node_response = account.transfer_with_outputs(transfer)
message_id = node_response['id']
print(f'Please check https://explorer.iota.org/devnet/addr/{message_id}')
