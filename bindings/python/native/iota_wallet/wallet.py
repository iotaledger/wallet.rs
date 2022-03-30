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
    def get_account_data(self, alias_index):
        """Get account data
        """
        # Setup the message
        message = {
            'cmd': 'GetAccount',
            'payload': alias_index,
        }

        return message

    @send_message_routine
    def get_accounts(self):
        """Get accounts
        """
        # Setup the message
        message = {
            'cmd': 'GetAccounts',
        }
        return message

    @send_message_routine
    def backup(self, destination, password):
        """Backup storage.
        """
        # Setup the message
        message = {
            'cmd': 'Backup',
            'payload': {
                'destination': destination,
                'password': password,
            }
        }
        return message

    @send_message_routine
    def restore_back(self, source, password):
        """Import accounts from storage.
        """
        # Setup the message
        message = {
            'cmd': 'RestoreBackup',
            'payload': {
                'source': source,
                'password': password,
            }
        }
        return message

    @send_message_routine
    def delete_storage(self):
        """Deletes the storage.
        """
        # Setup the message
        message = {
            'cmd': 'DeleteStorage',
        }
        return message

    @send_message_routine
    def generate_mnemonic(self):
        """Generates a new mnemonic.
        """
        # Setup the message
        message = {
            'cmd': 'GenerateMnemonic',
        }
        return message

    @send_message_routine
    def verify_mnemonic(self, mnemonic):
        """Checks if the given mnemonic is valid.
        """
        # Setup the message
        message = {
            'cmd': 'VerifyMnemonic',
            'payload': mnemonic,
        }
        return message

    @send_message_routine
    def set_client_options(self, client_options):
        """Updates the client options for all accounts.
        """
        # Setup the message
        message = {
            'cmd': 'SetClientOptions',
        }
        message['payload'] = client_options
        return message

    @send_message_routine
    def stop_background_sync(self):
        """Stop background syncing.
        """
        # Setup the message
        message = {
            'cmd': 'StopBackgroundSync',
        }
        return message

    @staticmethod
    def __return_str_or_none(str):
        if str:
            return str
        else:
            return None
