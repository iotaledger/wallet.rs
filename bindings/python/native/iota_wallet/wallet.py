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
        # Setup the message type
        message_type = {
            'cmd': 'CreateAccount',
            'payload': {
                'alias': self.__return_str_or_none(alias),
            }
        }
        message_type = dumps(message_type)

        # Send message to the Rust library
        response = iota_wallet.send_message(self.handle, message_type)
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
        # Setup the message type
        message_type = {
            'cmd': 'GetAccount',
            'payload': f'{alias_index}',
        }

        return message_type

    @send_message_routine
    def get_accounts(self):
        """Get accounts
        """
        # Setup the message type
        message_type = {
            'cmd': 'GetAccounts',
        }
        return message_type

    @send_message_routine
    def backup(self, destination, password):
        """Backup storage.
        """
        # Setup the message type
        message_type = {
            'cmd': 'Backup',
            'payload': {
                'destination': destination,
                'password': password,
            }
        }
        return message_type

    @send_message_routine
    def restore_back(self, source, password):
        """Import accounts from storage.
        """
        # Setup the message type
        message_type = {
            'cmd': 'RestoreBackup',
            'payload': {
                'source': source,
                'password': password,
            }
        }
        return message_type

    @send_message_routine
    def delete_storage(self):
        """Deletes the storage.
        """
        # Setup the message type
        message_type = {
            'cmd': 'DeleteStorage',
        }
        return message_type

    @send_message_routine
    def generate_mnemonic(self):
        """Generates a new mnemonic.
        """
        # Setup the message type
        message_type = {
            'cmd': 'GenerateMnemonic',
        }
        return message_type

    @send_message_routine
    def verify_mnemonic(self, mnemonic):
        """Checks if the given mnemonic is valid.
        """
        # Setup the message type
        message_type = {
            'cmd': 'VerifyMnemonic',
            'payload': mnemonic,
        }
        return message_type

    @send_message_routine
    def set_client_options(self, client_options):
        """Updates the client options for all accounts.
        """
        # Setup the message type
        message_type = {
            'cmd': 'SetClientOptions',
        }
        message_type['payload'] = client_options
        return message_type

    @send_message_routine
    def stop_background_sync(self):
        """Stop background syncing.
        """
        # Setup the message type
        message_type = {
            'cmd': 'StopBackgroundSync',
        }
        return message_type

    @staticmethod
    def __return_str_or_none(str):
        if str:
            return f'{str}'
        else:
            return None
