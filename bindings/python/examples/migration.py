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

# Migration
legacy_node = 'https://nodes.devnet.iota.org'
seed = 'TRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEEDTRYTESEED'
min_weight_magnitude = 9
permanode = 'https://chronicle.iota.org/api'


# Get the account data
address_index = 0
while True:
    migration_data = account_manager.get_migration_data(
        initial_address_index=address_index, nodes=[legacy_node], seed=seed, permanode=permanode)

    address_index += 30
    print(f'Is {migration_data["balance"]}i the correct balance?')
    print('Type N to search for more balance or type any other keys to exit.')
    if input() != 'N':
        break

# Send the migration bundle
input_address_indexes = []
for input in migration_data['inputs']:
    input_address_indexes.append(input['index'])
    try:
        bundle = account_manager.create_migration_bundle(
            seed, input_address_indexes, mine=True, timeout_seconds=40, offset=0)

        account_manager.send_migration_bundle(
            [legacy_node], bundle['bundle_hash'], min_weight_magnitude)

        hash = bundle['bundle_hash']
        print(f'Bundle sent, hash: {hash}')
    except ValueError as e:
        print(f'Error! {e}')
