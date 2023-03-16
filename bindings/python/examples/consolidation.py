# Copyright 2021 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
import requests
import json
import time


# Get the stronghold password, please get the password from your environment
# NEVER assign your password directly in the code!
STRONGHOLD_PASSWORD = "UNSAFEPASSWORD"

OUTPUT_CONSOLIDATION_THRESHOLD = 2

# note: `storage` and `storage_path` have to be declared together
account_manager = iw.AccountManager(
    storage_path='./alice-database',
    output_consolidation_threshold=OUTPUT_CONSOLIDATION_THRESHOLD,
    automatic_output_consolidation=False
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

address = account.addresses()[0]['address']['inner']


def get_funds(address):
    """Get the funds to the address
    """
    data = {
        'address': f'{address}'
    }
    headers = {'Content-Type': 'application/json',
               'Accept': 'application/json'}
    data = json.dumps(data)
    response = requests.post(
        'https://faucet.chrysalis-devnet.iota.cafe/api/plugins/faucet/enqueue', data=data, headers=headers)

    print(f'Response: {response.content}')


for _ in range(1, (OUTPUT_CONSOLIDATION_THRESHOLD + 1)):
    get_funds(address)

    # wait for the message to be confirmed
    time.sleep(10)


consolidated_outputs = account.consolidate_outputs(False)
print(f'Consolidated outputs: {consolidated_outputs}')
