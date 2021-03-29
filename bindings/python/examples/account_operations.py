# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# Create a AccountManager
manager = iota_wallet.AccountManager(
    storage_path='./storage')
manager.set_stronghold_password("password")
manager.store_mnemonic("Stronghold")

# First we'll create an example account and store it
client_options = {'node': 'https://api.lb-0.testnet.chrysalis2.com'}
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

# List messages
messages = account.list_messages(5, 0, message_type='Failed')
print(f'Messages: {messages}')
