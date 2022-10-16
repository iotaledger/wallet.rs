from iota_wallet import IotaWallet
import time

# In this example we will mint native tokens

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

transaction = account.create_alias_output(None, None)

# Wait a few seconds for the transaction to get confirmed
time.sleep(7)

account.sync_account()

native_token_options = {
    # 1000 hex encoded
    "circulatingSupply": "0x3e8",
    "maximumSupply": "0x3e8",
    "foundryMetadata": "0xab",
};

transaction = account.mint_native_token(native_token_options, None)

print(f'Sent transaction: {transaction}')
