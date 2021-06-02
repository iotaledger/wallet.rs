# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw
import os
from dotenv import load_dotenv
import json

# Create the log configuration, the log will be outputted in 'wallet.log'
log_config = {'color_enabled': True, "outputs": [
    {'name': './wallet.log', 'level_filter': 'debug'}]}
log_config_str = json.dumps(log_config)

# Init the logger
iw.init_logger(log_config_str)


# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

account_manager = iw.AccountManager(
    storage_path='./alice-database'
)  # note: `storage` and `storage_path` have to be declared together

account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# mnemonic (seed) should be set only for new storage
# once the storage has been initialized earlier then you should omit this step
account_manager.store_mnemonic("Stronghold")
