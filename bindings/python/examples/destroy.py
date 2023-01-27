import os
import shutil
import random
import time

import iota_wallet

# This example creates a wallet three times and destroys the AccountManager and deletes the storage if the path exists before

path = "static/user_wallets/test"
mnemonic = 'park target phrase cheese agent turn lunch wolf broccoli glance famous camp orient maid ribbon observe decrease cave subway possible hire puppy leader chronic'

def get(password):
    account_manager = iota_wallet.AccountManager(path)
    account_manager.set_stronghold_password(password)
    wallet = account_manager.get_account('Wallet')
    wallet.sync().execute()
    print(wallet.latest_address()["address"]["inner"])

def creating(password):
    if os.path.exists("static"):
        print("path exists")
        temp_account_manager = iota_wallet.AccountManager(path)
        temp_account_manager.stop_background_sync()
        time.sleep(1)
        temp_account_manager.destroy()
        shutil.rmtree(path)
    account_manager = iota_wallet.AccountManager(path)
    account_manager.set_stronghold_password(password)
    account_manager.store_mnemonic("Stronghold", mnemonic)
    client_options = {"nodes": [
        {
            "url": "https://api.lb-0.h.chrysalis-devnet.iota.cafe/",
            "auth": None,
            "disabled": False
        }
    ], "local_pow": True}
    account_initializer = account_manager.create_account(client_options)
    account_initializer.alias('Wallet')
    account: iota_wallet.AccountHandle = account_initializer.initialise()
    account.sync().execute()

password = "1234"
for i in range(3):
    try:
        shutil.rmtree(path)
    except Exception:
        pass
    password += random.choice("1234")
    creating(password)
    get(password)
    print(f"Get {i} worked")
