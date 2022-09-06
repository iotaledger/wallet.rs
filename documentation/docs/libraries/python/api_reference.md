<a id="iota_wallet"></a>

# iota\_wallet

<a id="iota_wallet.account"></a>

# iota\_wallet.account

<a id="iota_wallet.account.Account"></a>

## Account Objects

```python
class Account()
```

<a id="iota_wallet.account.Account.build_alias_output"></a>

#### build\_alias\_output

```python
def build_alias_output(amount, native_tokens, alias_id, state_index,
                       state_metadata, foundry_counter, unlock_conditions,
                       features, immutable_features)
```

Build alias output.

<a id="iota_wallet.account.Account.build_basic_output"></a>

#### build\_basic\_output

```python
def build_basic_output(amount, native_tokens, unlock_conditions, features)
```

Build basic output.

<a id="iota_wallet.account.Account.build_foundry_output"></a>

#### build\_foundry\_output

```python
def build_foundry_output(amount, native_tokens, serial_number, token_scheme,
                         unlock_conditions, features, immutable_features)
```

Build foundry output.

<a id="iota_wallet.account.Account.build_nft_output"></a>

#### build\_nft\_output

```python
def build_nft_output(amount, native_tokens, nft_id, unlock_conditions,
                     features, immutable_features)
```

BuildNftOutput.

<a id="iota_wallet.account.Account.burn_native_token"></a>

#### burn\_native\_token

```python
def burn_native_token(native_token, options)
```

Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
recommended to use melting, if the foundry output is available.

<a id="iota_wallet.account.Account.burn_nft"></a>

#### burn\_nft

```python
def burn_nft(nft_id, options)
```

Burn an nft output. Outputs controlled by it will be sweeped before if they don't have a storage
deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
burning, the foundry can never be destroyed anymore.

<a id="iota_wallet.account.Account.cnsolidate_outputs"></a>

#### cnsolidate\_outputs

```python
def cnsolidate_outputs(force, output_consolidation_threshold)
```

Consolidate outputs.

<a id="iota_wallet.account.Account.destroy_alias"></a>

#### destroy\_alias

```python
def destroy_alias(alias_id, options)
```

Destroy an alias output. Outputs controlled by it will be sweeped before if they don't have a
storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
sent to the governor address.

<a id="iota_wallet.account.Account.destroy_foundry"></a>

#### destroy\_foundry

```python
def destroy_foundry(foundry_id, options)
```

Destroy a foundry output with a circulating supply of 0.
Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias

<a id="iota_wallet.account.Account.generate_addresses"></a>

#### generate\_addresses

```python
def generate_addresses(amount, options=None)
```

Generate new addresses.

<a id="iota_wallet.account.Account.get_outputs_with_additional_unlock_conditions"></a>

#### get\_outputs\_with\_additional\_unlock\_conditions

```python
def get_outputs_with_additional_unlock_conditions(outputs_to_claim)
```

Get outputs with additional unlock conditions.

<a id="iota_wallet.account.Account.get_output"></a>

#### get\_output

```python
def get_output(output_id)
```

Get output.

<a id="iota_wallet.account.Account.get_transaction"></a>

#### get\_transaction

```python
def get_transaction(transaction_id)
```

Get transaction.

<a id="iota_wallet.account.Account.list_addresses"></a>

#### list\_addresses

```python
def list_addresses()
```

List addresses.

<a id="iota_wallet.account.Account.list_addresses_with_unspent_outputs"></a>

#### list\_addresses\_with\_unspent\_outputs

```python
def list_addresses_with_unspent_outputs()
```

Returns only addresses of the account with unspent outputs.

<a id="iota_wallet.account.Account.list_outputs"></a>

#### list\_outputs

```python
def list_outputs(filter_options=None)
```

Returns all outputs of the account.

<a id="iota_wallet.account.Account.list_unspent_outputs"></a>

#### list\_unspent\_outputs

```python
def list_unspent_outputs(filter_options=None)
```

Returns all unspent outputs of the account.

<a id="iota_wallet.account.Account.list_transactions"></a>

#### list\_transactions

```python
def list_transactions()
```

Returns all transaction of the account.

<a id="iota_wallet.account.Account.list_pending_transactions"></a>

#### list\_pending\_transactions

```python
def list_pending_transactions()
```

Returns all pending transaction of the account.

<a id="iota_wallet.account.Account.melt_native_token"></a>

#### melt\_native\_token

```python
def melt_native_token(native_token, options)
```

Melt native tokens. This happens with the foundry output which minted them, by increasing it's
`melted_tokens` field.

<a id="iota_wallet.account.Account.mint_native_token"></a>

#### mint\_native\_token

```python
def mint_native_token(native_token_options, options)
```

Mint native token.

<a id="iota_wallet.account.Account.minimum_required_storage_deposit"></a>

#### minimum\_required\_storage\_deposit

```python
def minimum_required_storage_deposit(output)
```

Minimum required storage deposit.

<a id="iota_wallet.account.Account.mint_nfts"></a>

#### mint\_nfts

```python
def mint_nfts(nfts_options, options)
```

Mint nfts.

<a id="iota_wallet.account.Account.get_balance"></a>

#### get\_balance

```python
def get_balance()
```

Get account balance information.

<a id="iota_wallet.account.Account.prepare_send_amount"></a>

#### prepare\_send\_amount

```python
def prepare_send_amount(addresses_with_amount, options)
```

Prepare send amount.

<a id="iota_wallet.account.Account.prepare_transaction"></a>

#### prepare\_transaction

```python
def prepare_transaction(outputs, options)
```

Prepare transaction.

<a id="iota_wallet.account.Account.sync_account"></a>

#### sync\_account

```python
def sync_account(options=None)
```

Sync the account by fetching new information from the nodes.
Will also retry pending transactions and consolidate outputs if necessary.

<a id="iota_wallet.account.Account.send_amount"></a>

#### send\_amount

```python
def send_amount(addresses_with_amount, options=None)
```

Send amount.

<a id="iota_wallet.account.Account.send_micro_transaction"></a>

#### send\_micro\_transaction

```python
def send_micro_transaction(addresses_with_micro_amount, options)
```

Send micro transaction.

<a id="iota_wallet.account.Account.send_native_tokens"></a>

#### send\_native\_tokens

```python
def send_native_tokens(addresses_native_tokens, options)
```

Send native tokens.

<a id="iota_wallet.account.Account.send_nft"></a>

#### send\_nft

```python
def send_nft(addresses_nft_ids, options)
```

Send nft.

<a id="iota_wallet.account.Account.set_alias"></a>

#### set\_alias

```python
def set_alias(alias)
```

Set alias.

<a id="iota_wallet.account.Account.send_transaction"></a>

#### send\_transaction

```python
def send_transaction(outputs, options)
```

Send a transaction.

<a id="iota_wallet.account.Account.sign_transaction_essence"></a>

#### sign\_transaction\_essence

```python
def sign_transaction_essence(prepared_transaction_data)
```

Sign a transaction essence.

<a id="iota_wallet.account.Account.submit_and_store_transaction"></a>

#### submit\_and\_store\_transaction

```python
def submit_and_store_transaction(signed_transaction_data)
```

Submit and store transaction.

<a id="iota_wallet.account.Account.try_claim_outputs"></a>

#### try\_claim\_outputs

```python
def try_claim_outputs(outputs_to_claim)
```

Try to claim outputs.

<a id="iota_wallet.account.Account.claim_outputs"></a>

#### claim\_outputs

```python
def claim_outputs(output_ids_to_claim)
```

Claim outputs.

<a id="iota_wallet.account.Account.send_outputs"></a>

#### send\_outputs

```python
@send_message_routine
def send_outputs(outputs, options=None)
```

Send outputs in a transaction.

<a id="iota_wallet.secret_manager"></a>

# iota\_wallet.secret\_manager

<a id="iota_wallet.secret_manager.LedgerNanoSecretManager"></a>

## LedgerNanoSecretManager Objects

```python
class LedgerNanoSecretManager(dict)
```

<a id="iota_wallet.secret_manager.LedgerNanoSecretManager.__init__"></a>

#### \_\_init\_\_

```python
def __init__(is_simulator)
```

Initialize a ledger nano secret manager.

<a id="iota_wallet.secret_manager.MnemonicSecretManager"></a>

## MnemonicSecretManager Objects

```python
class MnemonicSecretManager(dict)
```

<a id="iota_wallet.secret_manager.MnemonicSecretManager.__init__"></a>

#### \_\_init\_\_

```python
def __init__(mnemonic)
```

Initialize a mnemonic secret manager.

<a id="iota_wallet.secret_manager.StrongholdSecretManager"></a>

## StrongholdSecretManager Objects

```python
class StrongholdSecretManager(dict)
```

<a id="iota_wallet.secret_manager.StrongholdSecretManager.__init__"></a>

#### \_\_init\_\_

```python
def __init__(snapshot_path, password)
```

Initialize a stronghold secret manager.

<a id="iota_wallet.common"></a>

# iota\_wallet.common

<a id="iota_wallet.common.send_message_routine"></a>

#### send\_message\_routine

```python
def send_message_routine(func)
```

The routine of dump json string and call send_message()

<a id="iota_wallet.common.IotaWalletError"></a>

## IotaWalletError Objects

```python
class IotaWalletError(Exception)
```

iota-wallet error

<a id="iota_wallet.wallet"></a>

# iota\_wallet.wallet

<a id="iota_wallet.wallet.IotaWallet"></a>

## IotaWallet Objects

```python
class IotaWallet()
```

<a id="iota_wallet.wallet.IotaWallet.__init__"></a>

#### \_\_init\_\_

```python
def __init__(storage_path='./walletdb',
             client_options=None,
             coin_type=None,
             secret_manager=None)
```

Initialize the IOTA Wallet.

<a id="iota_wallet.wallet.IotaWallet.create_account"></a>

#### create\_account

```python
def create_account(alias=None)
```

Create a new account

<a id="iota_wallet.wallet.IotaWallet.get_account"></a>

#### get\_account

```python
def get_account(alias_index)
```

Get the account instance

<a id="iota_wallet.wallet.IotaWallet.get_account_data"></a>

#### get\_account\_data

```python
def get_account_data(alias_index)
```

Get account data

<a id="iota_wallet.wallet.IotaWallet.get_accounts"></a>

#### get\_accounts

```python
def get_accounts()
```

Get accounts

<a id="iota_wallet.wallet.IotaWallet.backup"></a>

#### backup

```python
def backup(destination, password)
```

Backup storage.

<a id="iota_wallet.wallet.IotaWallet.change_stronghold_password"></a>

#### change\_stronghold\_password

```python
def change_stronghold_password(password)
```

Change stronghold password.

<a id="iota_wallet.wallet.IotaWallet.clear_stronghold_password"></a>

#### clear\_stronghold\_password

```python
def clear_stronghold_password()
```

Clear stronghold password.

<a id="iota_wallet.wallet.IotaWallet.is_stronghold_password_available"></a>

#### is\_stronghold\_password\_available

```python
def is_stronghold_password_available()
```

Is stronghold password available.

<a id="iota_wallet.wallet.IotaWallet.recover_accounts"></a>

#### recover\_accounts

```python
def recover_accounts(account_gap_limit, address_gap_limit, sync_options)
```

Recover accounts.

<a id="iota_wallet.wallet.IotaWallet.remove_latest_account"></a>

#### remove\_latest\_account

```python
def remove_latest_account()
```

Remove latest account.

<a id="iota_wallet.wallet.IotaWallet.restore_back"></a>

#### restore\_back

```python
def restore_back(source, password)
```

Restore a backup from a Stronghold file
Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already created
If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a mnemonic was
stored, it will be gone.

<a id="iota_wallet.wallet.IotaWallet.delete_accounts_and_database"></a>

#### delete\_accounts\_and\_database

```python
def delete_accounts_and_database()
```

Deletes the accounts and database.

<a id="iota_wallet.wallet.IotaWallet.generate_mnemonic"></a>

#### generate\_mnemonic

```python
def generate_mnemonic()
```

Generates a new mnemonic.

<a id="iota_wallet.wallet.IotaWallet.verify_mnemonic"></a>

#### verify\_mnemonic

```python
def verify_mnemonic(mnemonic)
```

Checks if the given mnemonic is valid.

<a id="iota_wallet.wallet.IotaWallet.set_client_options"></a>

#### set\_client\_options

```python
def set_client_options(client_options)
```

Updates the client options for all accounts.

<a id="iota_wallet.wallet.IotaWallet.get_node_info"></a>

#### get\_node\_info

```python
def get_node_info(url, auth)
```

Get node info.

<a id="iota_wallet.wallet.IotaWallet.set_stronghold_password"></a>

#### set\_stronghold\_password

```python
def set_stronghold_password(password)
```

Set stronghold password.

<a id="iota_wallet.wallet.IotaWallet.set_stronghold_password_clear_interval"></a>

#### set\_stronghold\_password\_clear\_interval

```python
def set_stronghold_password_clear_interval(interval_in_milliseconds)
```

Set stronghold password clear interval.

<a id="iota_wallet.wallet.IotaWallet.store_mnemonic"></a>

#### store\_mnemonic

```python
def store_mnemonic(mnemonic)
```

Store mnemonic.

<a id="iota_wallet.wallet.IotaWallet.start_background_sync"></a>

#### start\_background\_sync

```python
def start_background_sync(options, interval_in_milliseconds)
```

Start background sync.

<a id="iota_wallet.wallet.IotaWallet.stop_background_sync"></a>

#### stop\_background\_sync

```python
def stop_background_sync()
```

Stop background syncing.

