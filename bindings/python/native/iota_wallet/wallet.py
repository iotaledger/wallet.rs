import iota_wallet
from iota_wallet.common import send_message_routine
from iota_wallet.account import Account
from json import loads, dumps


class IotaWallet():
    def __init__(self, storage_path='./walletdb', client_options=None, coin_type=None, secret_manager=None):
        """Initialize the IOTA Wallet.
        """

        # Setup the options
        options = {'storagePath': storage_path}
        if client_options:
            options['clientOptions'] = dumps(client_options)
        if coin_type:
            options['coinType'] = coin_type
        if secret_manager:
            options['secretManager'] = dumps(secret_manager)

        options = dumps(options)

        # Create the message handler
        self.handle = iota_wallet.create_message_handler(options)

    def get_handle(self):
        return self.handle

    def create_account(self, alias=None):
        """Create a new account
        """
        return self._send_cmd_routine(
            'CreateAccount', {
                'alias': self.__return_str_or_none(alias),
            }
        )

    def get_account(self, alias_index):
        """Get the account instance
        """
        return Account(alias_index, self.handle)

    @send_message_routine
    def _send_cmd_routine(self, cmd, payload=None):
        message = {
            'cmd': cmd
        }
        if payload:
            message['payload'] = payload
        return message

    def get_account_data(self, alias_index):
        """Get account data
        """
        return self._send_cmd_routine(
            'GetAccount',
            alias_index
        )

    def get_accounts(self):
        """Get accounts
        """
        return self._send_cmd_routine(
            'GetAccounts',
        )

    def backup(self, destination, password):
        """Backup storage.
        """
        return self._send_cmd_routine(
            'Backup', {
                'destination': destination,
                'password': password
            }
        )

    def change_stronghold_password(self, password):
        """Change stronghold password.
        """
        return self._send_cmd_routine(
            'ChangeStrongholdPassword', {
                'current_password': password,
                'new_password': password
            }
        )

    def clear_stronghold_password(self):
        """Clear stronghold password.
        """
        return self._send_cmd_routine(
            'ClearStrongholdPassword'
        )

    def is_stronghold_password_available(self):
        """Is stronghold password available.
        """
        return self._send_cmd_routine(
            'IsStrongholdPasswordAvailable'
        )

    def recover_accounts(self, account_gap_limit, address_gap_limit, sync_options):
        """Recover accounts.
        """
        return self._send_cmd_routine(
            'RecoverAccounts', {
                'account_gap_limit': account_gap_limit,
                'address_gap_limit': address_gap_limit,
                'sync_options': sync_options
            }
        )

    def remove_latest_account(self):
        """Remove latest account.
        """
        return self._send_cmd_routine(
            'RemoveLatestAccount'
        )

    def restore_back(self, source, password):
        """Import accounts from storage.
        """
        return self._send_cmd_routine(
            'RestoreBackup', {
                'source': source,
                'password': password
            }
        )

    def delete_accounts_and_database(self):
        """Deletes the accounts and database.
        """
        return self._send_cmd_routine(
            'DeleteAccountsAndDatabase'
        )

    def generate_mnemonic(self):
        """Generates a new mnemonic.
        """
        return self._send_cmd_routine(
            'GenerateMnemonic'
        )

    def verify_mnemonic(self, mnemonic):
        """Checks if the given mnemonic is valid.
        """
        return self._send_cmd_routine(
            'VerifyMnemonic',
            mnemonic
        )

    def set_client_options(self, client_options):
        """Updates the client options for all accounts.
        """
        return self._send_cmd_routine(
            'SetClientOptions',
            client_options
        )

    def get_node_info(self, url, auth):
        """Get node info.
        """
        return self._send_cmd_routine(
            'GetNodeInfo', {
                'url': url,
                'auth': auth
            }
        )

    def set_stronghold_password(self, password):
        """Set stronghold password.
        """
        return self._send_cmd_routine(
            'SetStrongholdPassword',
            password
        )

    def set_stronghold_password_clear_interval(self, interval_in_milliseconds):
        """Set stronghold password clear interval.
        """
        return self._send_cmd_routine(
            'SetStrongholdPasswordClearInterval', {
                'interval_in_milliseconds': interval_in_milliseconds
            }
        )

    def store_mnemonic(self, mnemonic):
        """Store mnemonic.
        """
        return self._send_cmd_routine(
            'StoreMnemonic',
            mnemonic
        )

    def start_background_sync(self, options, interval_in_milliseconds):
        """Start background sync.
        """
        return self._send_cmd_routine(
            'StartBackgroundSync', {
                'options': options,
                'interval_in_milliseconds': interval_in_milliseconds
            }
        )

    def stop_background_sync(self):
        """Stop background syncing.
        """
        return self._send_cmd_routine(
            'StopBackgroundSync',
        )

    @staticmethod
    def __return_str_or_none(str):
        if str:
            return str
        else:
            return None
