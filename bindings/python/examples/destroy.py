from iota_wallet import AccountHandle, AccountManager

# This example creates a wallet, destroys the AccountManager and recreates it again

path = "destroy-db"
mnemonic = 'park target phrase cheese agent turn lunch wolf broccoli glance famous camp orient maid ribbon observe decrease cave subway possible hire puppy leader chronic'

account_manager = AccountManager(path)
account_manager.set_stronghold_password("password")
account_manager.store_mnemonic("Stronghold", mnemonic)
client_options = {"nodes": [
    {
        "url": "https://api.lb-0.h.chrysalis-devnet.iota.cafe/",
        "auth": None,
        "disabled": False
    }
]}
account_initializer = account_manager.create_account(client_options)
account_initializer.alias('Wallet')
account: AccountHandle = account_initializer.initialise()
account.sync().execute()

account_manager.stop_background_sync()
account_manager.destroy()

account_manager = AccountManager(path)
wallet = account_manager.get_account('Wallet')
print(wallet.latest_address()["address"]["inner"])
