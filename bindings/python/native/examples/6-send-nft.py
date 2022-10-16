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
    "nftId": "0x17f97185f80fa56eab974de6b7bbb80fa812d4e8e37090d166a0a41da129cebc",
}];

transaction = account.send_nft(outputs)

print(f'Sent transaction: {transaction}')
