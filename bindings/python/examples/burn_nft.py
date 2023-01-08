from iota_wallet import IotaWallet
import time

# In this example we will burn native tokens

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

# TODO: replace with your own values.
nftId = "0xf95f4d5344217a2ba19a6c19a47f97d267edf8c4d76a7b8c08072ad35acbebbe"

# Send transaction.
t = account.burn_nft(token_id)

# Print transaction.
print(t)
