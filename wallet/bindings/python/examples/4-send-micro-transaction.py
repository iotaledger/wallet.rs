from iota_wallet import IotaWallet

# In this example we will send an amount below the minimum storage deposit

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

outputs = [{
    "address": "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu",
    "amount": "1",
}]

transaction = account.send_micro_transaction(outputs, None)

print(f'Sent transaction: {transaction}')
