from iota_wallet import IotaWallet

# This example restores the wallet from a stronghold.

client_options = {
    'nodes': ['https://api.testnet.shimmer.network'],
}

# Shimmer coin type
coin_type = 4219

wallet = IotaWallet('./restore-backup-database', client_options,
                    coin_type, 'Placeholder')


wallet.restore_backup("backup.stronghold", "some_hopefully_secure_password")

accounts = wallet.get_accounts()
print(f'Restored accounts: {accounts}')