# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw

# This example sends IOTA toens to an address.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

account = account_manager.get_account('Alice')
print(f'Account: {account.alias()} selected')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

for ac in account.list_messages():
    print(f"message {ac['id']}; confirmation status = {ac['confirmed']}'")
