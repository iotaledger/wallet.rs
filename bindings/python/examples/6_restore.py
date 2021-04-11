# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example restores a secured backup file.
account_manager = iw.AccountManager(
    storage_path='./alice-database-restored'
)

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

#  Add the path to the file from example 5-backup.js
#  for example: ./backup/2021-03-04T15-31-04-iota-wallet-backup-wallet.stronghold
backup_file_path = r'./backup/2021-03-31T14-45-23-iota-wallet-backup-wallet.stronghold'

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.import_accounts(backup_file_path, STRONGHOLD_PASSWORD)
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')
