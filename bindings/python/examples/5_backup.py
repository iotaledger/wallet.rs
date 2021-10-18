# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('SH_PASSWORD')

# This example backups your data in a secure file.
# You can move this file to another app or device and restore it.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

backup_dir_path = './backup'
if not os.path.exists(backup_dir_path):
    os.makedirs(backup_dir_path)
backup_file_path = account_manager.backup(backup_dir_path, STRONGHOLD_PASSWORD)

print(f'Backup path: {backup_file_path}')
