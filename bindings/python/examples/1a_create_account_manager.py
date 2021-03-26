import iota_wallet as iw

account_manager = iw.AccountManager(
    storage_path='./alice-database'
) #note: `storage` and `storage_path` have to be declared together

account_manager.set_stronghold_password("password")

# mnemonic (seed) should be set only for new storage
# once the storage has been initialized earlier then you should omit this step
account_manager.store_mnemonic("Stronghold")
