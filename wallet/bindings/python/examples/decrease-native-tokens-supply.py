from iota_wallet import IotaWallet

# In this example we will decrease the native token supply

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

# TODO: replace with your own values.
token_id = "0x08429fe5864378ce70699fc2d22bb144cb86a3c4833d136e3b95c5dadfd6ba0cef0500000000"
melt_amount = "0x20"

# Send transaction.
t = account.decrease_native_token_supply(token_id, melt_amount)

# Print transaction.
print(t)
