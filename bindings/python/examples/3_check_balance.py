# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw

# This example checks the account balance.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

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
