from iota_wallet import IotaWallet

# In this example we check if an output has only an address unlock condition and that the address is from the account.

wallet = IotaWallet("./alice-database")

account = wallet.get_account("Alice")

wallet.set_stronghold_password("some_hopefully_secure_password")

accountAddresses = account.addresses()

# using prepare_output
output = account.prepare_output(
    {
        "amount": "1000000",
        "recipientAddress": accountAddresses[0]['address'],
    }
)

def hexAddress(address):
    return wallet.bech32_to_hex(address['address'])

hexEncodedAccountAddresses = map(hexAddress, accountAddresses)

controlled_by_account = False

if len(output['unlockConditions']) == 1 and output['unlockConditions'][0]['type'] == 0:
    if output['unlockConditions'][0]['address']['pubKeyHash'] in hexEncodedAccountAddresses:
        controlled_by_account = True

print(
    f'The output has only an address unlock condition and that the address is from the account: {controlled_by_account}')
