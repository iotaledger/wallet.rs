# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

import threading
import time
import iota_wallet as iw
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

result_available = threading.Event()


def balance_changed_event_processing(event):
    print(f'On balanced changed: {event}')
    result_available.set()


# This example shows some events.
account_manager = iw.AccountManager(
    storage_path='./alice-database'
)

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
account_manager.set_stronghold_password(STRONGHOLD_PASSWORD)

account = account_manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# Get the latest unused address
last_address_obj = account.latest_address()
print(f"Address: {last_address_obj['address']}")

# Use the Chrysalis Faucet to send testnet tokens to your address:
print('Fill your address with the Faucet: https://faucet.tanglekit.de/')

iw.on_balance_change(balance_changed_event_processing)
print("Waiting for external event (on_balance_changed)...")

# wait for results to be available before continue
# will not wait longer than 360 seconds
result_available.wait(timeout=360)

print("Done.")
