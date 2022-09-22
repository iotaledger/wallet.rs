---
sidebar_label: account
title: iota_wallet.account
---

## Account Objects

```python
class Account()
```

### build\_alias\_output

```python
def build_alias_output(amount, native_tokens, alias_id, state_index,
                       state_metadata, foundry_counter, unlock_conditions,
                       features, immutable_features)
```

Build alias output.

### build\_basic\_output

```python
def build_basic_output(amount, native_tokens, unlock_conditions, features)
```

Build basic output.

### build\_foundry\_output

```python
def build_foundry_output(amount, native_tokens, serial_number, token_scheme,
                         unlock_conditions, features, immutable_features)
```

Build foundry output.

### build\_nft\_output

```python
def build_nft_output(amount, native_tokens, nft_id, unlock_conditions,
                     features, immutable_features)
```

Build NFT output.

### burn\_native\_token

```python
def burn_native_token(token_id, burn_amount, options=None)
```

Burn native tokens. This doesn&#x27;t require the foundry output which minted them, but will not increase
the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it&#x27;s
recommended to use melting, if the foundry output is available.

### burn\_nft

```python
def burn_nft(nft_id, options=None)
```

Burn an nft output. Outputs controlled by it will be sweeped before if they don&#x27;t have a storage
deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
burning, the foundry can never be destroyed anymore.

### consolidate\_outputs

```python
def consolidate_outputs(force, output_consolidation_threshold)
```

Consolidate outputs.

### destroy\_alias

```python
def destroy_alias(alias_id, options=None)
```

Destroy an alias output. Outputs controlled by it will be sweeped before if they don&#x27;t have a
storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
sent to the governor address.

### destroy\_foundry

```python
def destroy_foundry(foundry_id, options=None)
```

Destroy a foundry output with a circulating supply of 0.
Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias

### generate\_addresses

```python
def generate_addresses(amount, options=None)
```

Generate new addresses.

### get\_outputs\_with\_additional\_unlock\_conditions

```python
def get_outputs_with_additional_unlock_conditions(outputs_to_claim)
```

Get outputs with additional unlock conditions.

### get\_output

```python
def get_output(output_id)
```

Get output.

### get\_transaction

```python
def get_transaction(transaction_id)
```

Get transaction.

### list\_addresses

```python
def list_addresses()
```

List addresses.

### list\_addresses\_with\_unspent\_outputs

```python
def list_addresses_with_unspent_outputs()
```

Returns only addresses of the account with unspent outputs.

### list\_outputs

```python
def list_outputs(filter_options=None)
```

Returns all outputs of the account.

### list\_unspent\_outputs

```python
def list_unspent_outputs(filter_options=None)
```

Returns all unspent outputs of the account.

### list\_incoming\_transactions

```python
def list_incoming_transactions()
```

Returns all incoming transactions of the account.

### list\_transactions

```python
def list_transactions()
```

Returns all transactions of the account.

### list\_pending\_transactions

```python
def list_pending_transactions()
```

Returns all pending transactions of the account.

### decrease\_native\_token\_supply

```python
def decrease_native_token_supply(token_id, melt_amount, options=None)
```

Melt native tokens. This happens with the foundry output which minted them, by increasing it&#x27;s
`melted_tokens` field.

### increase\_native\_token\_supply

```python
def increase_native_token_supply(token_id,
                                 mint_amount,
                                 increase_native_token_supply_options=None,
                                 options=None)
```

Mint more native token.

### mint\_native\_token

```python
def mint_native_token(native_token_options, options=None)
```

Mint native token.

### minimum\_required\_storage\_deposit

```python
def minimum_required_storage_deposit(output)
```

Minimum required storage deposit.

### mint\_nfts

```python
def mint_nfts(nfts_options, options=None)
```

Mint nfts.

### get\_balance

```python
def get_balance()
```

Get account balance information.

### prepare\_send\_amount

```python
def prepare_send_amount(addresses_with_amount, options=None)
```

Prepare send amount.

### prepare\_transaction

```python
def prepare_transaction(outputs, options=None)
```

Prepare transaction.

### sync\_account

```python
def sync_account(options=None)
```

Sync the account by fetching new information from the nodes.
Will also retry pending transactions and consolidate outputs if necessary.

### send\_amount

```python
def send_amount(addresses_with_amount, options=None)
```

Send amount.

### send\_micro\_transaction

```python
def send_micro_transaction(addresses_with_micro_amount, options=None)
```

Send micro transaction.

### send\_native\_tokens

```python
def send_native_tokens(addresses_native_tokens, options=None)
```

Send native tokens.

### send\_nft

```python
def send_nft(addresses_nft_ids, options=None)
```

Send nft.

### set\_alias

```python
def set_alias(alias)
```

Set alias.

### send\_transaction

```python
def send_transaction(outputs, options=None)
```

Send a transaction.

### sign\_transaction\_essence

```python
def sign_transaction_essence(prepared_transaction_data)
```

Sign a transaction essence.

### submit\_and\_store\_transaction

```python
def submit_and_store_transaction(signed_transaction_data)
```

Submit and store transaction.

### claim\_outputs

```python
def claim_outputs(output_ids_to_claim)
```

Claim outputs.

### send\_outputs

```python
@send_message_routine
def send_outputs(outputs, options=None)
```

Send outputs in a transaction.

