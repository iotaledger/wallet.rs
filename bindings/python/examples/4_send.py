# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet

# This example sends IOTA toens to an address.
manager = iota_wallet.AccountManager(
    storage='Stronghold', storage_path='./alice-database')
manager.set_stronghold_password("password")
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

print(f"Available balance {account.balance()['available']}")

# TODO: Replace with the address of your choice!
transfer = iota_wallet.Transfer(amount=10000000,
                                address='atoi1qzt0nhsf38nh6rs4p6zs5knqp6psgha9wsv74uajqgjmwc75ugupx3y7x0r',
                                bench32_hrp=account.bech32_hrp(),
                                remainder_value_strategy='ReuseAddress')
node_response = synced.transfer(transfer)
print(
    f"Check your message on https://explorer.iota.org/chrysalis/message/{node_response['id']}")
