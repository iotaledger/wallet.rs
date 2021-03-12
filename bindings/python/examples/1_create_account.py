# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# This example creates a new database and account.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database')
manager.set_stronghold_password("password")
manager.store_mnemonic("Stronghold")

client_options = {
    'node': 'https://api.lb-0.testnet.chrysalis2.com', 'local_pow': True}
account_initialiser = manager.create_account(client_options)
account_initialiser.alias('Alice')
account = account_initialiser.initialise()

print(f'Account created: {account.alias()}')
