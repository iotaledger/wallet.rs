# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw

# This example restores a secured backup file.
account_manager = iw.AccountManager(
    storage_path='./alice-database-restored'
)
account_manager.set_stronghold_password("password")

#  Add the path to the file from example 5-backup.js
#  for example: ./backup/2021-03-04T15-31-04-iota-wallet-backup-wallet.stronghold
backup_file_path = r'./backup/2021-03-07T18-24-06-iota-wallet-backup-wallet.stronghold'

account_manager.import_accounts(backup_file_path, 'password')
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')
