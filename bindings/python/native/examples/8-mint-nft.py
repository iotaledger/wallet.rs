from iota_wallet import IotaWallet

# In this example we will mint an nft

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

outputs = [{
    "immutableMetadata": "some immutable nft metadata",
}];

transaction = account.mint_nfts(outputs)

print(f'Sent transaction: {transaction}')
