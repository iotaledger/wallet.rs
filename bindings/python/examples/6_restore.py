# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# This example restores a secured backup file.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database_backup')
manager.set_stronghold_password("password")

#  Add the path to the file from example 5-backup.js
#  for example: ./backup/2021-03-04T15-31-04-iota-wallet-backup-wallet.stronghold
backup_file_path = 'input your backup file'

manager.import_accounts(backup_file_path, 'password')
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')
