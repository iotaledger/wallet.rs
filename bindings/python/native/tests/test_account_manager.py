import iota_wallet as iw
import json
import datetime
import os

# Read the test vector
tv = dict()
with open('../../../tests/fixtures/test_vectors.json') as json_file:
    tv = json.load(json_file)
tv = tv['python']
pat = tv['account_manager']
account = None
manager = None
backup_file_path = None
mnemonic = None
created_time = int(datetime.datetime.now().timestamp())


def test_account_manager_basic_operations():
    global account, manager
    manager = iw.AccountManager(
        storage_path=pat['account_manager']['storage_path'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(pat['account_manager']['password'])
    manager.store_mnemonic(pat['account_manager']['store_mnemonic'])

    account_initialiser = manager.create_account(pat['client_options'])
    account_initialiser.alias(pat['alias'])
    account = account_initialiser.initialise()
    assert account.alias() == pat['alias']
    id = account.id()

    manager.remove_account(id)
    try:
        manager.get_account(pat['alias'])
    except ValueError as e:
        assert 'account not found' in str(e)

    account_initialiser = manager.create_account(pat['client_options'])
    account_initialiser.alias(pat['alias'])
    account = account_initialiser.initialise()
    assert account.alias() == pat['alias']


def test_account_manager_is_latest_address_unused():
    assert manager.is_latest_address_unused() == True


def test_account_manager_generate_mnemonic():
    global mnemonic
    mnemonic = manager.generate_mnemonic()
    assert isinstance(mnemonic, str)


def test_account_manager_verify_mnemonic():
    manager.verify_mnemonic(mnemonic)
    try:
        manager.verify_mnemonic('wrong_mnemonic')
    except ValueError as e:
        assert 'invalid mnemonic' in str(e)


def test_account_manager_get_accounts():
    assert isinstance(manager.get_accounts(), list)


def test_account_manager_reattach():
    id = account.id()
    try:
        manager.reattach(id, pat['message_id'])
    except ValueError as e:
        'message not found' in str(e)


def test_account_manager_promote():
    id = account.id()
    try:
        manager.promote(id, pat['message_id'])
    except ValueError as e:
        'message not found' in str(e)


def test_account_manager_retry():
    id = account.id()
    try:
        manager.retry(id, pat['message_id'])
    except ValueError as e:
        'message not found' in str(e)


def test_account_manager_get_balance_change_events():
    assert isinstance(manager.get_balance_change_events(), list)


def test_account_manager_get_balance_change_event_count():
    assert isinstance(manager.get_balance_change_event_count(), int)


def test_account_manager_get_transaction_confirmation_events():
    assert isinstance(manager.get_transaction_confirmation_events(), list)


def test_account_manager_get_transaction_confirmation_event_count():
    assert isinstance(manager.get_transaction_confirmation_event_count(), int)


def test_account_manager_get_new_transaction_events():
    assert isinstance(manager.get_new_transaction_events(), list)


def test_account_manager_get_new_transaction_event_count():
    assert isinstance(manager.get_new_transaction_event_count(), int)


def test_account_manager_get_reattachment_events():
    assert isinstance(manager.get_reattachment_events(), list)


def test_account_manager_get_reattachment_event_count():
    assert isinstance(manager.get_reattachment_event_count(), int)


def test_account_manager_get_broadcast_events():
    assert isinstance(manager.get_broadcast_events(), list)


def test_account_manager_get_broadcast_event_count():
    assert isinstance(manager.get_broadcast_event_count(), int)


def test_account_manager_internal_transfer():
    try:
        manager.internal_transfer('Alice', 'Jason', 100)
    except ValueError as e:
        'account not found' in str(e)


def test_account_manager_backup_and_restore():
    global manager, backup_file_path

    # Backup
    manager = iw.AccountManager(
        storage_path=pat['backup_restore']['account_manager']['storage_path'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(
        pat['backup_restore']['account_manager']['password'])
    manager.store_mnemonic(pat['account_manager']['store_mnemonic'])

    account_initialiser = manager.create_account(
        tv['account_manager']['client_options'])
    account_initialiser.alias(tv['account_manager']['alias'])
    account = account_initialiser.initialise()

    backup_dir_path = pat['backup_restore']['backup_dir_path']
    if not os.path.exists(backup_dir_path):
        os.makedirs(backup_dir_path)
    backup_file_path = manager.backup(
        backup_dir_path, pat['backup_restore']['account_manager']['password'])

    # Restore
    manager = iw.AccountManager(
        storage_path=pat['backup_restore']['account_manager']['storage_path_backup'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(
        pat['backup_restore']['account_manager']['password'])
    manager.import_accounts(
        backup_file_path, pat['backup_restore']['account_manager']['password'])

    account = manager.get_account(pat['backup_restore']['alias'])
    assert account.alias() == pat['backup_restore']['alias']


def test_account_manager_import_accounts():
    manager = iw.AccountManager(
        storage_path=pat['backup_restore']['account_manager']['storage_path_2nd_backup'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_storage_password(
        pat['backup_restore']['account_manager']['storage_password'])
    manager.import_accounts(
        backup_file_path, pat['backup_restore']['account_manager']['password'])

    account = manager.get_account(pat['backup_restore']['alias'])
    assert account.alias() == pat['backup_restore']['alias']


def test_account_manager_start_background_sync():
    manager.start_background_sync(30, True)


def test_account_manager_stop_background_sync():
    manager.stop_background_sync()
