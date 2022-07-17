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
                'accountId': self.alias_index,
                'method': {
                    'name': method,
                }
            }
        }
        if data:
            message['payload']['method']['data'] = data

        return message

    def build_alias_output(self,
                           amount,
                           native_tokens,
                           alias_id,
                           state_index,
                           state_metadata,
                           foundry_counter,
                           unlock_conditions,
                           features,
                           immutable_features):
        """Build alias output.
        """
        return self._call_account_method(
            'BuildAliasOutput', {
                'amount': amount,
                'native_tokens': native_tokens,
                'alias_id': alias_id,
                'state_index': state_index,
                'state_metadata': state_metadata,
                'foundry_counter': foundry_counter,
                'unlock_conditions': unlock_conditions,
                'features': features,
                'immutable_features': immutable_features
            }
        )

    def build_basic_output(self,
                           amount,
                           native_tokens,
                           unlock_conditions,
                           features):
        """Build basic output.
        """
        return self._call_account_method(
            'BuildBasicOutput', {
                'amount': amount,
                'native_tokens': native_tokens,
                'unlock_conditions': unlock_conditions,
                'features': features
            }
        )

    def build_foundry_output(self,
                             amount,
                             native_tokens,
                             serial_number,
                             token_scheme,
                             unlock_conditions,
                             features,
                             immutable_features):
        """Build foundry output.
        """
        return self._call_account_method(
            'BuildFoundryOutput', {
                'amount': amount,
                'native_tokens': native_tokens,
                'serial_number': serial_number,
                'token_scheme': token_scheme,
                'unlock_conditions': unlock_conditions,
                'features': features,
                'immutable_features': immutable_features
            }
        )

    def build_nft_output(self,
                         amount,
                         native_tokens,
                         nft_id,
                         unlock_conditions,
                         features,
                         immutable_features):
        """BuildNftOutput.
        """
        return self._call_account_method(
            'BuildNftOutput', {
                'amount': amount,
                'native_tokens': native_tokens,
                'nft_id': nft_id,
                'unlock_conditions': unlock_conditions,
                'features': features,
                'immutable_features': immutable_features
            }
        )

    def cnsolidate_outputs(self,
                           force,
                           output_consolidation_threshold):
        """Consolidate outputs.
        """
        return self._call_account_method(
            'ConsolidateOutputs', {
                'force': force,
                'output_consolidation_threshold':  output_consolidation_threshold
            }
        )

    def generate_addresses(self, amount, options=None):
        """Generate new addresses.
        """
        return self._call_account_method(
            'GenerateAddresses', {
                'amount': amount,
                'options': options
            }
        )

    def get_outputs_with_additional_unlock_conditions(self, outputs_to_claim):
        """Get outputs with additional unlock conditions.
        """
        return self._call_account_method(
            'GetOutputsWithAdditionalUnlockConditions', {
                'outputs_to_claim': outputs_to_claim
            }
        )

    def get_output(self, output_id):
        """Get output.
        """
        return self._call_account_method(
            'GetOutput', {
                'output_id': output_id
            }
        )

    def get_transaction(self, transaction_id):
        """Get transaction.
        """
        return self._call_account_method(
            'GetTransaction', {
                'transaction_id': transaction_id
            }
        )

    def list_addresses(self):
        """List addresses.
        """
        return self._call_account_method(
            'ListAddresses'
        )

    def list_addresses_with_unspent_outputs(self):
        """Returns only addresses of the account with unspent outputs.
        """
        return self._call_account_method(
            'ListAddressesWithUnspentOutputs'
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

    def mint_native_token(self, native_token_options, options):
        """Mint native token.
        """
        return self._call_account_method(
            'MintNativeToken', {
                'native_token_options': native_token_options,
                'options': options
            }
        )

    def minimum_required_storage_deposit(self, output):
        """Minimum required storage deposit.
        """
        return self._call_account_method(
            'MinimumRequiredStorageDeposit', {
                'output': output
            }
        )

    def mint_nfts(self, nfts_options, options):
        """Mint nfts.
        """
        return self._call_account_method(
            'MintNfts', {
                'nfts_options': nfts_options,
                'options': options
            }
        )

    def get_balance(self):
        """Get account balance information.
        """
        return self._call_account_method(
            'GetBalance'
        )

    def prepare_send_amount(self, addresses_with_amount, options):
        """Prepare send amount.
        """
        return self._call_account_method(
            'PrepareSendAmount', {
                'addresses_with_amount': addresses_with_amount,
                'options': options
            }
        )

    def prepare_transaction(self, outputs, options):
        """Prepare transaction.
        """
        return self._call_account_method(
            'PrepareTransaction', {
                'outputs': outputs,
                'options': options
            }
        )

    def sync_account(self, options=None):
        """Sync the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        return self._call_account_method(
            'SyncAccount', {
                'options': options,
            }
        )

    def send_amount(self, addresses_with_amount, options):
        """Send amount.
        """
        return self._call_account_method(
            'SendAmount', {
                'addresses_with_amount': addresses_with_amount,
                'options': options
            }
        )

    def send_micro_transaction(self, addresses_with_micro_amount, options):
        """Send micro transaction.
        """
        return self._call_account_method(
            'SendMicroTransaction', {
                'addresses_with_micro_amount': addresses_with_micro_amount,
                'options': options
            }
        )

    def send_native_tokens(self, addresses_native_tokens, options):
        """Send native tokens.
        """
        return self._call_account_method(
            'SendNativeTokens', {
                'addresses_native_tokens': addresses_native_tokens,
                'options': options
            }
        )

    def send_nft(self, addresses_nft_ids, options):
        """Send nft.
        """
        return self._call_account_method(
            'SendNft', {
                'addresses_nft_ids': addresses_nft_ids,
                'options': options
            }
        )

    def set_alias(self, alias):
        """Set alias.
        """
        return self._call_account_method(
            'SetAlias', {
                'alias': alias

            }
        )

    def send_transaction(self, outputs, options):
        """Send a transaction.
        """
        return self._call_account_method(
            'SendTransaction', {
                'outputs': outputs,
                'options': options
            }
        )

    def sign_transaction_essence(self, prepared_transaction_data):
        """Sign a transaction essence.
        """
        return self._call_account_method(
            'SignTransactionEssence', {
                'prepared_transaction_data': prepared_transaction_data

            }
        )

    def submit_and_store_transaction(self, signed_transaction_data):
        """Submit and store transaction.
        """
        return self._call_account_method(
            'SubmitAndStoreTransaction', {
                'signed_transaction_data': signed_transaction_data

            }
        )

    def try_claim_outputs(self, outputs_to_claim):
        """Try to claim outputs.
        """
        return self._call_account_method(
            'TryClaimOutputs', {
                'outputs_to_claim': outputs_to_claim

            }
        )

    def claim_outputs(self, output_ids_to_claim):
        """Claim outputs.
        """
        return self._call_account_method(
            'ClaimOutputs', {
                'output_ids_to_claim': output_ids_to_claim

            }
        )

    @ send_message_routine
    def send_outputs(self, outputs, options=None):
        """Send outputs in a transaction.
        """
        return self._call_account_method(
            'SendOutputs', {
                'outputs': outputs,
                'options': options,
            }
        )
