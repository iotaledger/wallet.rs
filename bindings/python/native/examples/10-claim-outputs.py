from iota_wallet import IotaWallet

# In this example we will claim outputs that have additional unlock conditions as expiration or storage deposit return

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

wallet.set_stronghold_password("some_hopefully_secure_password")

# Sync account with the node
response = account.sync_account()

# Only the unspent outputs in the account
output_ids = account.get_outputs_with_additional_unlock_conditions('All')

print(f'Available outputs to claim: {output_ids}')

transaction = account.claim_outputs(output_ids)

print(f'Sent transaction: {transaction}')
