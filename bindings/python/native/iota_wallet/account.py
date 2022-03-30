from iota_wallet.common import send_message_routine


class Account:
    def __init__(self, alias_index, handle):
        self.alias_index = alias_index
        self.handle = handle

    @send_message_routine
    def __str__(self):
        message = {
            'cmd': 'GetAccount',
            'payload': self.alias_index,
        }
        return message

    @send_message_routine
    def _call_account_method(self, method, data=None):
        message = {
            'cmd': 'CallAccountMethod',
            'payload': {
                'account_id': self.alias_index,
                'method': {
                    'name': method,
                }
            }
        }
        if data:
            message['payload']['method']['data'] = data

        return message

    def generate_addresses(self, amount, options=None):
        """Generate new unused addresses.
        """
        return self._call_account_method(
            'GenerateAddresses', {
                'amount': amount,
                'options': options
            }
        )

    def list_addresses(self):
        """List addresses.
        """
        return self._call_account_method(
            'ListAddresses'
        )

    def list_addresses_with_balance(self):
        """Returns only addresses of the account with balance.
        """
        return self._call_account_method(
            'ListAddressesWithBalance'
        )

    def list_outputs(self):
        """Returns all outputs of the account.
        """
        return self._call_account_method(
            'ListOutputs'
        )

    def list_unspent_outputs(self):
        """Returns all unspent outputs of the account.
        """
        return self._call_account_method(
            'ListUnspentOutputs'
        )

    def list_transactions(self):
        """Returns all transaction of the account.
        """
        return self._call_account_method(
            'ListTransactions'
        )

    def list_pending_transactions(self):
        """Returns all pending transaction of the account.
        """
        return self._call_account_method(
            'ListPendingTransactions'
        )

    def get_balance(self):
        """Get account balance information.
        """
        return self._call_account_method(
            'GetBalance'
        )

    def sync_account(self, options=None):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        return self._call_account_method(
            'SyncAccount', {
                'options': options,
            }
        )

    @ send_message_routine
    def send_transfer(self, outputs, options=None):
        """Syncs the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        return self._call_account_method(
            'SendTransfer', {
                'outputs': outputs,
                'options': options,
            }
        )
