from iota_wallet.common import send_message_routine
from json import dumps


class CallAccountMethod:
    @send_message_routine
    def generate_addresses(self, account_id, amount, options=None):
        """Generate new unused addresses.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'GenerateAddresses',
                    'data': {
                        'amount': amount,
                    },
                    'options': options
                }
            }
        }

        return message_type

    @send_message_routine
    def list_addresses(self, account_id):
        """List addresses.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListAddresses',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_addresses_with_balance(self, account_id):
        """Returns only addresses of the account with balance.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListAddressesWithBalance',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_outputs(self, account_id):
        """Returns all outputs of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListOutputs',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_unspent_outputs(self, account_id):
        """Returns all unspent outputs of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListUnspentOutputs',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_transactions(self, account_id):
        """Returns all transaction of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListTransactions',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_pending_transactions(self, account_id):
        """Returns all pending transaction of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'ListPendingTransactions',
                },
            }
        }
        return message_type

    @send_message_routine
    def get_balance(self, account_id):
        """Get account balance information.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'GetBalance',
                },
            }
        }
        return message_type

    @send_message_routine
    def sync_account(self, account_id, options):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'SyncAccount',
                    'data': options,
                },
            }
        }
        return message_type

    @send_message_routine
    def send_transfer(self, account_id, outputs, options=None):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': f'{account_id}',
                'method': {
                    'name': 'SendTransfer',
                    'data':  {
                        'outputs': outputs,
                        'options': options,
                    }
                }
            }
        }
        return message_type
