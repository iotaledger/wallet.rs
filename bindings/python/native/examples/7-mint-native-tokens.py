from iota_wallet import IotaWallet

# In this example we will mint native tokens

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

transaction = account.create_alias_output()

# Get the transaction confirmed
account.retry_until_included(transaction.block_id)

account.sync_account()

outputs = [{
    "circulatingSupply": "10",
    "maximumSupply": "10",
    "foundryMetadata": "0xab",
}];

transaction = account.mint_native_token(outputs)

print(f'Sent transaction: {transaction}')
