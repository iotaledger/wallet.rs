from iota_wallet import IotaWallet

# This example checks the balance of an account.

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync()
print(f'Synced: {response}')

# Just calculate the balance with the known state
balance = account.get_balance()
print(f'Balance: {balance}')
