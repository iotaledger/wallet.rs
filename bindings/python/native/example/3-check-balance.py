from iota_wallet import IotaWallet

# This example checks the balance of a account.

client_options = {
    'primaryNode': {'url': 'https://api.lb-0.h.chrysalis-devnet.iota.cafe'},
    'localPow': True,
}
secret_manager = ("flame fever pig forward exact dash body idea link scrub tennis minute " +
          "surge unaware prosper over waste kitten ceiling human knife arch situate civil")

wallet = IotaWallet('./alice-database', client_options, secret_manager)
account = wallet.get_account('Alice')
print(f'Account: {account}')

response = account.sync_account()
print(f'Syncing... {response}')

balance = account.get_balance()
print(f'Balance: {balance}')
