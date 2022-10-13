from iota_wallet import IotaWallet

# In this example we will send native tokens

wallet = IotaWallet('./alice-database')

account = wallet.get_account('Alice')

# Sync account with the node
response = account.sync_account()
print(f'Synced: {response}')

wallet.set_stronghold_password("some_hopefully_secure_password")

outputs = [{
    "address": "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu",
    "nativeTokens": [{
        "tokenId": "0x08847bd287c912fadedb6bf38900bda9f2d377b75b2a0bece8738699f56ebca4130100000000",
        "amount": "1"
    }],
}];

transaction = account.send_native_tokens(outputs)

print(f'Sent transaction: {transaction}')
