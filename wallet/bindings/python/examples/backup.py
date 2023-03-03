from iota_wallet import IotaWallet, StrongholdSecretManager

# This example creates an account and then a stronghold backup.

client_options = {
    'nodes': ['https://api.testnet.shimmer.network'],
}

# Shimmer coin type
coin_type = 4219

secret_manager = StrongholdSecretManager(
    "wallet.stronghold", "some_hopefully_secure_password")

wallet = IotaWallet('./backup-database', client_options,
                    coin_type, secret_manager)

# Store the mnemonic in the Stronghold snapshot, this only needs to be done once
account = wallet.store_mnemonic("float thumb deer talent charge shrug mixed often desert caution mean roof" +
                                " game shed hint victory opinion verb cloud area pony yellow since motion")

accounts = wallet.create_account('Alice')

wallet.backup("backup.stronghold", "some_hopefully_secure_password")
print(f'Created backup')
