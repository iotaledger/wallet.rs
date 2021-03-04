# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet
import os

# This example backups your data in a secure file.
# You can move this file to another app or device and restore it.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database')
manager.set_stronghold_password("password")

backup_dir_path = './backup'
if not os.path.exists(backup_dir_path):
    os.makedirs(backup_dir_path)
backup_file_path = manager.backup(backup_dir_path)
print(f'Backup path: {backup_file_path}')
