
# Copyright 2022 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

from iota_wallet import IotaWallet, MnemonicSecretManager


def test_address_generation_iota():

    client_options = {
        'nodes': [],
    }

    # IOTA coin type
    coin_type = 4218

    secret_manager = MnemonicSecretManager(
        "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast")

    wallet = IotaWallet('./test_address_generation_iota',
                        client_options, coin_type, secret_manager)

    wallet.create_account('Alice')

    account = wallet.get_account('Alice')

    addresses = account.addresses()

    assert 'smr1qpg2xkj66wwgn8p2ggnp7p582gj8g6p79us5hve2tsudzpsr2ap4sp36wye' == addresses[
        0]['address']


def test_address_generation_shimmer():

    client_options = {
        'nodes': [],
    }

    # Shimmer coin type
    coin_type = 4219

    secret_manager = MnemonicSecretManager(
        "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast")

    wallet = IotaWallet('./test_address_generation_shimmer',
                        client_options, coin_type, secret_manager)

    wallet.create_account('Alice')

    account = wallet.get_account('Alice')

    addresses = account.addresses()

    assert 'smr1qzev36lk0gzld0k28fd2fauz26qqzh4hd4cwymlqlv96x7phjxcw6ckj80y' == addresses[
        0]['address']
