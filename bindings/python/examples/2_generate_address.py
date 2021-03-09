# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# This example genrates a new address.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database')
manager.set_stronghold_password("password")
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

address = account.generate_address()
print(f'New address: {address}')

# You can also get the latest unused address
last_address_obj = account.latest_address()
print(f"Address: {last_address_obj['address']}")

# Use the Chrysalis Faucet to send testnet tokens to your address:
print('Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/')
