# ... continue from prev example 1a

# general Tangle specific options
client_options = {
    'node': 'https://api.lb-0.testnet.chrysalis2.com',
    'local_pow': True
}

# an account is generated with the given alias via `account_initialiser`
account_initialiser = account_manager.create_account(client_options)
account_initialiser.alias('Alice')

# initialise account based via `account_initialiser`
# store it to db and sync with Tangle
account = account_initialiser.initialise()
print(f'Account created: {account.alias()}')
