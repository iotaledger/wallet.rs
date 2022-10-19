---
sidebar_label: wallet
title: iota_wallet.wallet
---

## IotaWallet Objects

```python
class IotaWallet()
```

### \_\_init\_\_

```python
def __init__(storage_path='./walletdb',
             client_options=None,
             coin_type=None,
             secret_manager=None)
```

Initialize the IOTA Wallet.

### create\_account

```python
def create_account(alias=None)
```

Create a new account

### get\_account

```python
def get_account(alias_index)
```

Get the account instance

### get\_account\_data

```python
def get_account_data(alias_index)
```

Get account data

### get\_accounts

```python
def get_accounts()
```

Get accounts

### backup

```python
def backup(destination, password)
```

Backup storage.

### change\_stronghold\_password

```python
def change_stronghold_password(password)
```

Change stronghold password.

### clear\_stronghold\_password

```python
def clear_stronghold_password()
```

Clear stronghold password.

### is\_stronghold\_password\_available

```python
def is_stronghold_password_available()
```

Is stronghold password available.

### recover\_accounts

```python
def recover_accounts(account_start_index, account_gap_limit, address_gap_limit,
                     sync_options)
```

Recover accounts.

### remove\_latest\_account

```python
def remove_latest_account()
```

Remove latest account.

### restore\_back

```python
def restore_backup(source, password)
```

Restore a backup from a Stronghold file
Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already created
If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a mnemonic was
stored, it will be gone.

### delete\_accounts\_and\_database

```python
def delete_accounts_and_database()
```

Deletes the accounts and database.

### generate\_mnemonic

```python
def generate_mnemonic()
```

Generates a new mnemonic.

### verify\_mnemonic

```python
def verify_mnemonic(mnemonic)
```

Checks if the given mnemonic is valid.

### set\_client\_options

```python
def set_client_options(client_options)
```

Updates the client options for all accounts.

### get\_node\_info

```python
def get_node_info(url, auth)
```

Get node info.

### set\_stronghold\_password

```python
def set_stronghold_password(password)
```

Set stronghold password.

### set\_stronghold\_password\_clear\_interval

```python
def set_stronghold_password_clear_interval(interval_in_milliseconds)
```

Set stronghold password clear interval.

### store\_mnemonic

```python
def store_mnemonic(mnemonic)
```

Store mnemonic.

### start\_background\_sync

```python
def start_background_sync(options, interval_in_milliseconds)
```

Start background sync.

### stop\_background\_sync

```python
def stop_background_sync()
```

Stop background syncing.

