from iota_wallet.common import send_message_routine


class Account:
    def __init__(self, alias_index, handle):
        self.alias_index = alias_index
        self.handle = handle

    @send_message_routine
    def __str__(self):
        message_type = {
            'cmd': 'GetAccount',
            'payload': self.alias_index,
        }
        return message_type

    @send_message_routine
    def generate_addresses(self, amount, options=None):
        """Generate new unused addresses.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'GenerateAddresses',
                    'data': {
                        'amount': amount,
                        'options': options
                    },
                }
            }
        }

        return message_type

    @send_message_routine
    def list_addresses(self):
        """List addresses.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListAddresses',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_addresses_with_balance(self):
        """Returns only addresses of the account with balance.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListAddressesWithBalance',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_outputs(self):
        """Returns all outputs of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListOutputs',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_unspent_outputs(self):
        """Returns all unspent outputs of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListUnspentOutputs',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_transactions(self):
        """Returns all transaction of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListTransactions',
                },
            }
        }
        return message_type

    @send_message_routine
    def list_pending_transactions(self):
        """Returns all pending transaction of the account.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'ListPendingTransactions',
                },
            }
        }
        return message_type

    @send_message_routine
    def get_balance(self):
        """Get account balance information.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'GetBalance',
                },
            }
        }
        return message_type

    @send_message_routine
    def sync_account(self, options=None):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': 'SyncAccount',
                    'data': {
                        'options': options,
                    }
                },
            }
        }
        return message_type

    @send_message_routine
    def send_transfer(self, outputs, options=None):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        # Setup the message type
        message_type = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
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
