# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# This example checks the account balance.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database')
manager.set_stronghold_password("password")
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

print(f"Available balance {account.balance()['available']}")
