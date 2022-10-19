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
            options['clientOptions'] = client_options
        if coin_type:
            options['coinType'] = coin_type
        if secret_manager:
            options['secretManager'] = secret_manager

        options = dumps(options)

        # Create the message handler
        self.handle = iota_wallet.create_message_handler(options)

    def get_handle(self):
        return self.handle

    def create_account(self, alias=None):
        """Create a new account
        """
        return self._send_cmd_routine(
            'createAccount', {
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
            'getAccount',
            alias_index
        )

    def get_accounts(self):
        """Get accounts
        """
        return self._send_cmd_routine(
            'getAccounts',
        )

    def backup(self, destination, password):
        """Backup storage.
        """
        return self._send_cmd_routine(
            'backup', {
                'destination': destination,
                'password': password
            }
        )

    def change_stronghold_password(self, password):
        """Change stronghold password.
        """
        return self._send_cmd_routine(
            'changeStrongholdPassword', {
                'currentPassword': password,
                'newPassword': password
            }
        )

    def clear_stronghold_password(self):
        """Clear stronghold password.
        """
        return self._send_cmd_routine(
            'clearStrongholdPassword'
        )

    def is_stronghold_password_available(self):
        """Is stronghold password available.
        """
        return self._send_cmd_routine(
            'isStrongholdPasswordAvailable'
        )

    def recover_accounts(self, account_start_index, account_gap_limit, address_gap_limit, sync_options):
        """Recover accounts.
        """
        return self._send_cmd_routine(
            'recoverAccounts', {
                'accountStartIndex': account_start_index,
                'accountGapLimit': account_gap_limit,
                'addressGapLimit': address_gap_limit,
                'syncOptions': sync_options
            }
        )

    def remove_latest_account(self):
        """Remove latest account.
        """
        return self._send_cmd_routine(
            'removeLatestAccount'
        )

    def restore_backup(self, source, password):
        """Restore a backup from a Stronghold file
           Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already created
           If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a mnemonic was
           stored, it will be gone.
        """
        return self._send_cmd_routine(
            'restoreBackup', {
                'source': source,
                'password': password
            }
        )

    def generate_mnemonic(self):
        """Generates a new mnemonic.
        """
        return self._send_cmd_routine(
            'generateMnemonic'
        )

    def verify_mnemonic(self, mnemonic):
        """Checks if the given mnemonic is valid.
        """
        return self._send_cmd_routine(
            'verifyMnemonic',
            mnemonic
        )

    def set_client_options(self, client_options):
        """Updates the client options for all accounts.
        """
        return self._send_cmd_routine(
            'setClientOptions',
            client_options
        )

    def get_node_info(self, url, auth):
        """Get node info.
        """
        return self._send_cmd_routine(
            'getNodeInfo', {
                'url': url,
                'auth': auth
            }
        )

    def set_stronghold_password(self, password):
        """Set stronghold password.
        """
        return self._send_cmd_routine(
            'setStrongholdPassword',
            password
        )

    def set_stronghold_password_clear_interval(self, interval_in_milliseconds):
        """Set stronghold password clear interval.
        """
        return self._send_cmd_routine(
            'setStrongholdPasswordClearInterval', {
                'intervalInMilliseconds': interval_in_milliseconds
            }
        )

    def store_mnemonic(self, mnemonic):
        """Store mnemonic.
        """
        return self._send_cmd_routine(
            'storeMnemonic',
            mnemonic
        )

    def start_background_sync(self, options, interval_in_milliseconds):
        """Start background sync.
        """
        return self._send_cmd_routine(
            'startBackgroundSync', {
                'options': options,
                'intervalInMilliseconds': interval_in_milliseconds
            }
        )

    def stop_background_sync(self):
        """Stop background syncing.
        """
        return self._send_cmd_routine(
            'stopBackgroundSync',
        )

    @staticmethod
    def __return_str_or_none(str):
        if str:
            return str
        else:
            return None
