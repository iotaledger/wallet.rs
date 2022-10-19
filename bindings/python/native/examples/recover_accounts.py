from iota_wallet import IotaWallet, StrongholdSecretManager

# This example searches for accounts with unspent outputs.

client_options = {
    'nodes': ['https://api.testnet.shimmer.network'],
}

# Shimmer coin type
coin_type = 4219

secret_manager = StrongholdSecretManager(
    "wallet.stronghold", "some_hopefully_secure_password")

wallet = IotaWallet('./alice-database', client_options,
                    coin_type, secret_manager)

# Store the mnemonic in the Stronghold snapshot, this only needs to be done once
account = wallet.store_mnemonic("flame fever pig forward exact dash body idea link scrub tennis minute " +
                                "surge unaware prosper over waste kitten ceiling human knife arch situate civil")

# Searches for unspent outputs until no ones are found for 3 accounts in a row
# and checks the addresses for each account until 10 addresses in a row have nothing
accounts = wallet.recover_accounts(0, 3, 10, None)
print(accounts)
