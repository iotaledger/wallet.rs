from iota_wallet import IotaWallet

# In this example we will send an nft

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

outputs = [{
    "address": "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu",
    "nftId": "0xe192461b30098a5da889ef6abc9e8130bf3b2d980450fa9201e5df404121b932",
}];

transaction = account.send_nft(outputs)

print(f'Sent transaction: {transaction}')
