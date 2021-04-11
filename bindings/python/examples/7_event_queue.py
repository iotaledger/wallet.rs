# Copyright 2020 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0


import iota_wallet
import threading
import queue
import time
import os
from dotenv import load_dotenv

# Load the env variables
load_dotenv()

# Get the stronghold password
STRONGHOLD_PASSWORD = os.getenv('STRONGHOLD_PASSWORD')

# This example shows how to listen to on_balance_change event.

# The queue to store received events
q = queue.Queue()


def worker():
    """The worker to process the queued events.
    """
    while True:
        item = q.get(True)
        print(f'Get event: {item}')
        q.task_done()


def balance_changed_event_processing(event):
    """Processing function when event is received.
    """
    print(f'On balanced changed: {event}')
    q.put(event)


# Get the acount manager
manager = iota_wallet.AccountManager(
    storage_path='./alice-database')

# NOTE: In real use cases you need to set the password in a safer way, like getting it from env variables
manager.set_stronghold_password(STRONGHOLD_PASSWORD)

# Get the account
account = manager.get_account('Alice')
print(f'Account: {account.alias()}')

# Always sync before doing anything with the account
print('Syncing...')
synced = account.sync().execute()

# Get the latest unused address
last_address_obj = account.latest_address()
print(f"Address: {last_address_obj['address']}")

# turn-on the worker thread
threading.Thread(target=worker, daemon=True).start()

# listen to the on_balance_change event
iota_wallet.on_balance_change(balance_changed_event_processing)

# Use the Chrysalis Faucet to send testnet tokens to your address:
print(
    f"Fill your Address ({last_address_obj['address']['inner']}) with the Faucet: https://faucet.tanglekit.de/")
print("To see how the on_balance_change is called, please send tokens to the address in 1 min")
time.sleep(60)

# block until all tasks are done
q.join()
print('All work completed')
