from iota_wallet import IotaWallet

# In this example we will list the sent transactions

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

wallet.set_stronghold_password("some_hopefully_secure_password")

# All transactions sent from the the account
transactions = account.transactions()
print(f'Transactions: {transactions}')

# Pending transactions
pending_transactions = account.pending_transactions()
print(f'Pending transactions: {pending_transactions}')
