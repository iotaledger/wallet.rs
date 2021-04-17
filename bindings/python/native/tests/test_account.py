import iota_wallet as iw
import json
import datetime
import os
import pytest

# Read the test vector
tv = dict()
with open('tests/fixtures/test_vectors.json') as json_file:
    tv = json.load(json_file)
account = None
manager = None
synced_account = None
created_time = int(datetime.datetime.now().timestamp())


"""
Test Account Initialiser
"""


def test_account_initialiser():
    global manager, account
    pat = tv['account_initialiser']
    manager = iw.AccountManager(
        storage_path=pat['account_manager']['storage_path'])

    # NOTE: In real use cases, it is necessary to get the password form the env variables or other safer ways!
    manager.set_stronghold_password(pat['account_manager']['password'])
    manager.store_mnemonic(pat['account_manager']['store_mnemonic'])
    account_initialiser = manager.create_account(pat['client_options'])
    account_initialiser.signer_type(pat['signer_type'])
    account_initialiser.alias(pat['alias'])
    account_initialiser.created_at(created_time)
    account_initialiser.messages([])
    account_initialiser.addresses([])

    account = account_initialiser.initialise()
    assert account.alias() == pat['alias']
    assert account.created_at() == created_time


"""
Test Account Handle
"""


def test_account_handle_sync():
    account.sync().execute()


def test_account_handle_transfer():
    pat = tv['transfer']
    transfer = iw.Transfer(amount=pat['amount'],
                           address=pat['address'],
                           remainder_value_strategy=pat['remainder_value_strategy'])

    try:
        node_response = account.transfer(transfer)
        # Should be insufficient funds
        assert False
    except ValueError as e:
        assert 'insufficient funds' in str(e)

def test_account_handle_transfer_with_outputs():
    pat = tv['transfer']
    transfer_outputs = [{ "address": pat['address'], "amount": pat['amount'] }]
    transfer = iw.TransferWithOutputs(outputs=transfer_outputs,
                           remainder_value_strategy=pat['remainder_value_strategy'])

    try:
        node_response = account.transfer_with_outputs(transfer)
        # Should be insufficient funds
        assert False
    except ValueError as e:
        assert 'insufficient funds' in str(e)

def test_account_handle_retry():
    message_id = tv['account_handle']['message_id']
    try:
        account.retry(message_id)
    except ValueError as e:
        assert 'message not found' in str(e)


def test_account_handle_promote():
    message_id = tv['account_handle']['message_id']
    try:
        account.promote(message_id)
    except ValueError as e:
        assert 'message not found' in str(e)


def test_account_handle_reattach():
    message_id = tv['account_handle']['message_id']
    try:
        account.reattach(message_id)
    except ValueError as e:
        assert 'message not found' in str(e)


def test_account_handle_id():
    id = account.id()
    assert isinstance(id, str)


def test_account_handle_signer_type():
    signer_type = account.signer_type()
    assert signer_type == tv['account_handle']['signer_type']


def test_account_handle_index():
    index = account.index()
    assert isinstance(index, int)


def test_account_handle_alias():
    alias = account.alias()
    assert alias == tv['account_handle']['alias']


def test_account_handle_created_at():
    fetched_created_time = account.created_at()
    assert fetched_created_time == created_time


def test_account_handle_last_synced_at():
    last_synced_time = account.last_synced_at()
    assert last_synced_time > created_time


def test_account_handle_client_options():
    client_options = account.client_options()
    assert client_options == tv['account_handle']['client_options']


def test_account_handle_bech32_hrp():
    bech32_hrp = account.bech32_hrp()
    assert isinstance(bech32_hrp, str)


def test_account_handle_consolidate_outputs():
    consolidated_outputs = account.consolidate_outputs()
    assert isinstance(consolidated_outputs, list)


def test_account_handle_generate_address():
    generated_address = account.generate_address()
    assert isinstance(generated_address,
                      dict) and 'address' in generated_address


def test_account_handle_get_unused_address():
    unused_address = account.get_unused_address()
    assert isinstance(unused_address, dict) and 'address' in unused_address


def test_account_handle_is_latest_address_unused():
    is_latest_address_unused = account.is_latest_address_unused()
    assert isinstance(is_latest_address_unused, bool)


def test_account_handle_latest_address():
    latest_address = account.latest_address()
    assert isinstance(latest_address, dict) and 'address' in latest_address


def test_account_handle_addresses():
    addresses = account.addresses()
    assert isinstance(addresses, list) and 'address' in addresses[0]


def test_account_handle_balance():
    balance = account.balance()
    assert isinstance(balance, dict) and 'available' in balance


def test_account_handle_set_alias():
    account.set_alias(tv['account_handle']['updated_alias'])
    alias = account.alias()
    assert alias == tv['account_handle']['updated_alias']


@pytest.mark.skip(reason="https://github.com/iotaledger/wallet.rs/issues/527")
def test_account_handle_set_client_options():
    account.set_client_options(tv['account_handle']['updated_client_options'])
    client_options = account.client_options()
    assert client_options == tv['account_handle']['updated_client_options']


def test_account_handle_message_count():
    message_count = account.message_count()
    assert isinstance(message_count, int)


def test_account_handle_list_messages():
    messages = account.list_messages()
    assert isinstance(messages, list)


def test_account_handle_list_spent_addresses():
    spent_addresses = account.list_spent_addresses()
    assert isinstance(spent_addresses, list)


def test_account_handle_list_unspent_addresses():
    unspent_addresses = account.list_unspent_addresses()
    assert isinstance(unspent_addresses,
                      list) and 'address' in unspent_addresses[0]


def test_account_handle_get_message():
    message_id = tv['account_handle']['message_id']
    messages = account.get_message(message_id)
    assert messages == None


def test_account_synchronizer():
    global synced_account
    account_synchronizer = account.sync()
    account_synchronizer.gap_limit(20)
    account_synchronizer.skip_persistence()
    account_synchronizer.address_index(10)

    # Note: the synced_account should not be used direclty in python binding
    synced_account = account_synchronizer.execute()


"""
Test Synced Account
"""


def test_synced_account_account_handle():
    account_handle = synced_account.account_handle()
    alias = account.alias()
    assert alias == tv['account_handle']['updated_alias']


def test_synced_account_deposit_address():
    address = synced_account.deposit_address()
    assert isinstance(address, dict) and 'address' in address


def test_synced_account_messages():
    messages = synced_account.messages()
    assert isinstance(messages, list)


def test_synced_account_addresses():
    addresses = synced_account.addresses()
    assert isinstance(addresses, list)


"""
TestAccounts Synchronizer
"""


def test_accounts_synchronizer():
    accounts_synchronizer = manager.sync_accounts()
    accounts_synchronizer.gap_limit(20)
    accounts_synchronizer.address_index(0)
    try:
        accounts_synchronizer.execute()
    except ValueError as e:
        assert 'Failed to find seed vault' in str(e)
