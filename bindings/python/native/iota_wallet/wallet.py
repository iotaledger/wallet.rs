import iota_wallet
from iota_wallet.common import send_message_routine
from iota_wallet.account import Account
from json import loads, dumps


class IotaWallet():
    def __init__(self, storage_folder='./walletdb', client_options=None, signer=None):
        """Initialize the IOTA Wallet.
        """

        # Setup the options
        options = {'storageFolder': storage_folder}
        if client_options:
            options['clientOptions'] = dumps(client_options)
        if signer:
            options['signer'] = dumps({'Mnemonic': signer})

        options = dumps(options)

        # Create the message handler
        self.handle = iota_wallet.create_message_handler(options)

    def get_handle(self):
        return self.handle

    def create_account(self, alias=None):
        # Setup the message
        message = {
            'cmd': 'CreateAccount',
            'payload': {
                'alias': self.__return_str_or_none(alias),
            }
        }
        message = dumps(message)

        # Send message to the Rust library
        response = iota_wallet.send_message(self.handle, message)
        response = loads(response)
        account_id = response['payload']['index']

        return self.get_account(account_id)

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

    def restore_back(self, source, password):
        """Import accounts from storage.
        """
        return self._send_cmd_routine(
            'RestoreBackup', {
                'source': source,
                'password': password
            }
        )

    def delete_storage(self):
        """Deletes the storage.
        """
        return self._send_cmd_routine(
            'DeleteStorage'
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
