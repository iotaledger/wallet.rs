import iota_wallet
import json
import os

# Read the test vector
tv = dict()
with open('tests/fixtures/test_vectors.json') as json_file:
    tv = json.load(json_file)
account = None
manager = None


def test_creat_acocunt_manager():
    global account
    for pat in tv['account_manager']:
        manager = iota_wallet.AccountManager(
            storage=pat['account_manager']['storage'], storage_path=pat['account_manager']['storage_path'])

        # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
        manager.set_stronghold_password(pat['account_manager']['password'])
        manager.store_mnemonic(pat['account_manager']['store_mnemonic'])
        account_initialiser = manager.create_account(pat['client_options'])
        account_initialiser.alias(pat['alias'])
        account = account_initialiser.initialise()
        assert account.alias() == pat['alias']
        break


# def test_address():
#     synced = account.sync().execute()
#     address = account.generate_address()
#     last_address_obj = account.latest_address()
#     assert isinstance(last_address_obj, dict) and 'address' in last_address_obj


# def test_balance():
#     account_balance = account.balance()
#     assert isinstance(account_balance, dict) and 'available' in account_balance


# def test_transfer():
#     synced = account.sync().execute()
#     pat = tv['transfer']
#     transfer = iota_wallet.Transfer(amount=pat['amount'],
#                                     address=pat['address'],
#                                     bench32_hrp=account.bech32_hrp(),
#                                     remainder_value_strategy=pat['remainder_value_strategy'])
#     try:
#         node_response = account.transfer(transfer)
#         # Should be insufficient funds
#         assert False
#     except ValueError as e:
#         assert 'insufficient funds' in str(e)


def test_backup_and_restore():
    global manager
    pat = tv['backup_restore']

    # Backup
    manager = iota_wallet.AccountManager(
        storage=pat['account_manager']['store_mnemonic'], storage_path=pat['account_manager']['storage_path'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(pat['account_manager']['password'])

    backup_dir_path = pat['backup_dir_path']
    if not os.path.exists(backup_dir_path):
        os.makedirs(backup_dir_path)
    backup_file_path = manager.backup(backup_dir_path)

    # Restore
    manager = iota_wallet.AccountManager(
        storage=pat['account_manager']['storage'], storage_path=pat['account_manager']['storage_path_backup'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(pat['account_manager']['password'])
    manager.import_accounts(
        backup_file_path, pat['account_manager']['password'])

    account = manager.get_account(pat['alias'])
    assert account.alias() == pat['alias']


def test_accounts_synchronizer():
    accounts_synchronizer = manager.sync_accounts()
    accounts_synchronizer.gap_limit(20)
    accounts_synchronizer.address_index(0)
    try:
        accounts_synchronizer.execute()
    except ValueError as e:
        assert 'Failed to find seed vault' in str(e)
