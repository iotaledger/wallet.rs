from iota_wallet.common import send_message_routine


class Account:
    def __init__(self, alias_index, handle):
        self.alias_index = alias_index
        self.handle = handle

    @send_message_routine
    def __str__(self):
        message = {
            'cmd': 'getAccount',
            'payload': self.alias_index,
        }
        return message

    @send_message_routine
    def _call_account_method(self, method, data=None):
        message = {
            'cmd': 'callAccountMethod',
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
            'buildAliasOutput', {
                'amount': amount,
                'nativeTokens': native_tokens,
                'aliasId': alias_id,
                'stateIndex': state_index,
                'stateMetadata': state_metadata,
                'foundryCounter': foundry_counter,
                'unlockConditions': unlock_conditions,
                'features': features,
                'immutableFeatures': immutable_features
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
            'buildBasicOutput', {
                'amount': amount,
                'nativeTokens': native_tokens,
                'unlockConditions': unlock_conditions,
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
            'buildFoundryOutput', {
                'amount': amount,
                'nativeTokens': native_tokens,
                'serialNumber': serial_number,
                'tokenScheme': token_scheme,
                'unlockConditions': unlock_conditions,
                'features': features,
                'immutableFeatures': immutable_features
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
            'buildNftOutput', {
                'amount': amount,
                'nativeTokens': native_tokens,
                'nftId': nft_id,
                'unlockConditions': unlock_conditions,
                'features': features,
                'immutableFeatures': immutable_features
            }
        )

    def burn_native_token(self,
                          token_id,
                          burn_amount,
                          options=None):
        """Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
        the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
        recommended to use melting, if the foundry output is available.
        """
        return self._call_account_method(
            'burnNativeToken', {
                'tokenId': token_id,
                'burnAmount': burn_amount,
                'options':  options
            }
        )

    def burn_nft(self,
                 nft_id,
                 options=None):
        """Burn an nft output. Outputs controlled by it will be swept before if they don't have a storage
        deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
        burning, the foundry can never be destroyed anymore.
        """
        return self._call_account_method(
            'burnNft', {
                'nftId': nft_id,
                'options':  options
            }
        )

    def consolidate_outputs(self,
                            force,
                            output_consolidation_threshold):
        """Consolidate outputs.
        """
        return self._call_account_method(
            'consolidateOutputs', {
                'force': force,
                'outputConsolidationThreshold':  output_consolidation_threshold
            }
        )

    def create_alias_output(self,
                            alias_output_options,
                            options):
        """Create an alias output.
        """
        return self._call_account_method(
            'createAliasOutput', {
                'aliasOutputOptions': alias_output_options,
                'options':  options
            }
        )

    def destroy_alias(self,
                      alias_id,
                      options=None):
        """Destroy an alias output. Outputs controlled by it will be swept before if they don't have a
        storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
        sent to the governor address.
        """
        return self._call_account_method(
            'destroyAlias', {
                'aliasId': alias_id,
                'options':  options
            }
        )

    def destroy_foundry(self,
                        foundry_id,
                        options=None):
        """Destroy a foundry output with a circulating supply of 0.
        Native tokens in the foundry (minted by other foundries) will be transacted to the controlling alias
        """
        return self._call_account_method(
            'destroyFoundry', {
                'foundryId': foundry_id,
                'options':  options
            }
        )

    def generate_addresses(self, amount, options=None):
        """Generate new addresses.
        """
        return self._call_account_method(
            'generateAddresses', {
                'amount': amount,
                'options': options
            }
        )

    def get_outputs_with_additional_unlock_conditions(self, outputs_to_claim):
        """Get outputs with additional unlock conditions.
        """
        return self._call_account_method(
            'getOutputsWithAdditionalUnlockConditions', {
                'outputsToClaim': outputs_to_claim
            }
        )

    def get_output(self, output_id):
        """Get output.
        """
        return self._call_account_method(
            'getOutput', {
                'outputId': output_id
            }
        )

    def get_transaction(self, transaction_id):
        """Get transaction.
        """
        return self._call_account_method(
            'getTransaction', {
                'transactionId': transaction_id
            }
        )

    def addresses(self):
        """List addresses.
        """
        return self._call_account_method(
            'addresses'
        )

    def addresses_with_unspent_outputs(self):
        """Returns only addresses of the account with unspent outputs.
        """
        return self._call_account_method(
            'addressesWithUnspentOutputs'
        )

    def outputs(self, filter_options=None):
        """Returns all outputs of the account.
        """
        return self._call_account_method(
            'outputs', {
                'filterOptions': filter_options
            }
        )

    def unspent_outputs(self, filter_options=None):
        """Returns all unspent outputs of the account.
        """
        return self._call_account_method(
            'unspentOutputs', {
                'filterOptions': filter_options
            }
        )

    def incoming_transactions(self):
        """Returns all incoming transactions of the account.
        """
        return self._call_account_method(
            'incomingTransactions'
        )

    def transactions(self):
        """Returns all transaction of the account.
        """
        return self._call_account_method(
            'transactions'
        )

    def pending_transactions(self):
        """Returns all pending transactions of the account.
        """
        return self._call_account_method(
            'pendingTransactions'
        )

    def decrease_native_token_supply(self,
                                     token_id,
                                     melt_amount,
                                     options=None):
        """Melt native tokens. This happens with the foundry output which minted them, by increasing it's
        `melted_tokens` field.
        """
        return self._call_account_method(
            'decreaseNativeTokenSupply', {
                'tokenId': token_id,
                'meltAmount': melt_amount,
                'options':  options
            }
        )

    def increase_native_token_supply(self, token_id, mint_amount, increase_native_token_supply_options=None, options=None):
        """Mint more native token.
        """
        return self._call_account_method(
            'increaseNativeTokenSupply', {
                'tokenId': token_id,
                'mintAmount': mint_amount,
                'increaseNativeTokenSupplyOptions': increase_native_token_supply_options,
                'options': options
            }
        )

    def mint_native_token(self, native_token_options, options=None):
        """Mint native token.
        """
        return self._call_account_method(
            'mintNativeToken', {
                'nativeTokenOptions': native_token_options,
                'options': options
            }
        )

    def minimum_required_storage_deposit(self, output):
        """Minimum required storage deposit.
        """
        return self._call_account_method(
            'minimumRequiredStorageDeposit', {
                'output': output
            }
        )

    def mint_nfts(self, nfts_options, options=None):
        """Mint nfts.
        """
        return self._call_account_method(
            'mintNfts', {
                'nftsOptions': nfts_options,
                'options': options
            }
        )

    def get_balance(self):
        """Get account balance information.
        """
        return self._call_account_method(
            'getBalance'
        )

    def prepare_send_amount(self, addresses_with_amount, options=None):
        """Prepare send amount.
        """
        return self._call_account_method(
            'prepareSendAmount', {
                'addressesWithAmount': addresses_with_amount,
                'options': options
            }
        )

    def prepare_transaction(self, outputs, options=None):
        """Prepare transaction.
        """
        return self._call_account_method(
            'prepareTransaction', {
                'outputs': outputs,
                'options': options
            }
        )

    def sync_account(self, options=None):
        """Sync the account by fetching new information from the nodes.
           Will also retry pending transactions and consolidate outputs if necessary.
        """
        return self._call_account_method(
            'syncAccount', {
                'options': options,
            }
        )

    def send_amount(self, addresses_with_amount, options=None):
        """Send amount.
        """
        return self._call_account_method(
            'sendAmount', {
                'addressesWithAmount': addresses_with_amount,
                'options': options
            }
        )

    def send_micro_transaction(self, addresses_with_micro_amount, options=None):
        """Send micro transaction.
        """
        return self._call_account_method(
            'sendMicroTransaction', {
                'addressesWithMicroAmount': addresses_with_micro_amount,
                'options': options
            }
        )

    def send_native_tokens(self, addresses_native_tokens, options=None):
        """Send native tokens.
        """
        return self._call_account_method(
            'sendNativeTokens', {
                'addressesNativeTokens': addresses_native_tokens,
                'options': options
            }
        )

    def send_nft(self, addresses_nft_ids, options=None):
        """Send nft.
        """
        return self._call_account_method(
            'sendNft', {
                'addressesAndNftIds': addresses_nft_ids,
                'options': options
            }
        )

    def set_alias(self, alias):
        """Set alias.
        """
        return self._call_account_method(
            'setAlias', {
                'alias': alias

            }
        )

    def sign_transaction_essence(self, prepared_transaction_data):
        """Sign a transaction essence.
        """
        return self._call_account_method(
            'signTransactionEssence', {
                'preparedTransactionData': prepared_transaction_data

            }
        )

    def submit_and_store_transaction(self, signed_transaction_data):
        """Submit and store transaction.
        """
        return self._call_account_method(
            'submitAndStoreTransaction', {
                'signedTransactionData': signed_transaction_data

            }
        )

    def claim_outputs(self, output_ids_to_claim):
        """Claim outputs.
        """
        return self._call_account_method(
            'claimOutputs', {
                'outputIdsToClaim': output_ids_to_claim

            }
        )

    @ send_message_routine
    def send_outputs(self, outputs, options=None):
        """Send outputs in a transaction.
        """
        return self._call_account_method(
            'sendOutputs', {
                'outputs': outputs,
                'options': options,
            }
        )
