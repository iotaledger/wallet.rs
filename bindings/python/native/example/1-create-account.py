from iota_wallet import IotaWallet

# This example creates a new database and account

client_options = {
    'primaryNode': {'url': 'https://api.lb-0.h.chrysalis-devnet.iota.cafe'},
    'localPow': True,
}
secret_manager = ("flame fever pig forward exact dash body idea link scrub tennis minute " +
          "surge unaware prosper over waste kitten ceiling human knife arch situate civil")

wallet = IotaWallet('./alice-database', client_options, secret_manager)
account = wallet.create_account('Alice')
print(account)
