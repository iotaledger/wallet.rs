# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('SH_PASSWORD')


# Create a AccountManager
manager = iota_wallet.AccountManager(
    storage_path='./storage')

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
manager.set_stronghold_password(STRONGHOLD_PASSWORD)
manager.store_mnemonic("Stronghold")

# First we'll create an example account and store it
client_options = {
    "nodes": [
        {
            "url": "https://api.lb-0.h.chrysalis-devnet.iota.cafe",
            "auth": None,
            "disabled": False
        }
    ],
    "local_pow": True
}
account_initialiser = manager.create_account(client_options)
account_initialiser.alias('alias')
account = account_initialiser.initialise()

# Update alias
account.set_alias('the new alias')

# Get unspent addresses
unspend_addresses = account.list_unspent_addresses()
print(f'Unspend addresses: {unspend_addresses}')

# Get spent addresses
spent_addresses = account.list_spent_addresses()
print(f'Spent addresses: {spent_addresses}')

# Get all addresses
addresses = account.addresses()
print(f'All addresses: {addresses}')

# Generate a new unused address
new_address = account.generate_address()
print(f'New address: {new_address}')

# Generate multiple new unused addresses
new_addresses = account.generate_addresses(2)
print(f'New addresses: {new_addresses}')

# List messages
messages = account.list_messages(5, 0, message_type='Failed')
print(f'Messages: {messages}')
