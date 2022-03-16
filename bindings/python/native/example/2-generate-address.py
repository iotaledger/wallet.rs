from iota_wallet import IotaWallet

# This example genrates a new address.

client_options = {
    'primaryNode': {'url': 'https://api.lb-0.h.chrysalis-devnet.iota.cafe'},
    'localPow': True,
}
signer = ("flame fever pig forward exact dash body idea link scrub tennis minute " +
          "surge unaware prosper over waste kitten ceiling human knife arch situate civil")

wallet = IotaWallet('./alice-database', client_options, signer)
account = wallet.get_account('Alice')
print(f'Account: {account}')

# Always sync before doing anything with the account
response = account.sync_account()
# response = account.sync_account({'syncAllAddresses': True})
print(f'Syncing... {response}')

addresses = account.generate_addresses(5)
# addresses = account.generate_addresses(
#     2, {'internal': True, 'metadata': {'syncing': True, 'network': 'Testnet'}})
print(f'Addresses: {addresses}')
