# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet as iw

# This example sends IOTA toens to an address.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)
account_manager.set_stronghold_password("password")

print("Selecting a specific account")
account = account_manager.get_account('Alice')
print(f'Account: {account.alias()} selected')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

print(f"Available balance {account.balance()['available']}")

# TODO: Replace with the address of your choice!
transfer = iw.Transfer(
    amount=1_000_000,
    address='atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r',
    remainder_value_strategy='ReuseAddress'
)

# Propogate the Transfer to Tangle
# and get a response from the Tangle
node_response = account.transfer(transfer)
print(
    node_response
)
