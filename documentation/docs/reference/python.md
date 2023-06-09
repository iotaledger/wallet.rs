---
description: Official IOTA Wallet Library Software Python API reference.
image: /img/logo/logo_dark.svg
keywords:
- api
- python
- param
- type
- reference
---
# Python API Reference

:::info
The following APIs will throw an exception if an error occurs.
For all optional values, the default values are the same as in the [Rust API](rust.md).
:::

## init_logger(config)

| Param  | Type  | Default     | Description              |
| ------ | ----- | ----------- | ------------------------ |
| config | `str` | `undefined` | The logger configuration |

The config is the dumped string from the JSON, which key:value pairs are from the [fern logger](https://github.com/iotaledger/common-rs/blob/main/fern-logger/src/config.rs).

Please check the `example/logger_example.py` to see how to use it.

## AccountManager

### constructor(storage_path (optional), password (optional), polling_interval (optional), automatic_output_consolidation(optional), output_consolidation_threshold(optional), sync_spent_outputs(optional), persist_events(optional)): [AccountManager](#accountmanager)

Creates a new instance of the AccountManager.

| Param                                  | Type      | Default       | Description                                                                                    |
| -------------------------------------- | --------- | ------------- | ---------------------------------------------------------------------------------------------- |
| [storage_path]                         | `str`     | ``./storage`` | The path where the database file will be saved                                                 |
| [storage_password]                     | `str`     | `undefined`   | The storage password to encrypt/decrypt accounts                                               |
| [polling_interval]                     | `int`     | `30000`       | The polling interval in seconds                                                                |
| [automatic_output_consolidation]       | `bool`    | `true`        | Disables the automatic output consolidation process                                            |
| [output_consolidation_threshold]       | `int`     | `100`         | Sets the number of outputs an address must have to trigger the automatic consolidation process |
| [sync_spent_outputs]                   | `boolean` | `false`       | Enables fetching spent output history on account sync                                          |
| [persist_events]                       | `boolean` | `false`       | Enables event persistence                                                                      |
| [allow_create_multiple_empty_accounts] | `boolean` | `false`       | Enables creating accounts with latest account being empty                                      |

:::info
If the _storage_path_ is set, then the _storage_ needs to be set too. An exception will be thrown when errors happened.
:::

Returns the constructed [AccountManager](#accountmanager).

### start_background_sync(polling_interval, automatic_output_consolidation): void

Starts the background polling and MQTT monitoring.

| Param                          | Type      | Default     | Description                                      |
| ------------------------------ | --------- | ----------- | ------------------------------------------------ |
| polling_interval               | `number`  | `undefined` | The polling interval in seconds                  |
| automatic_output_consolidation | `boolean` | `undefined` | If outputs should get consolidated automatically |

### stop_background_sync(): void

Stops the background polling and MQTT monitoring.

### set_storage_password(password): void

Sets the password used for encrypting the storage.

| Param    | Type  | Default     | Description          |
| -------- | ----- | ----------- | -------------------- |
| password | `str` | `undefined` | The storage password |

### set_stronghold_password(password): void

Sets the Stronghold password.

| Param    | Type  | Default     | Description          |
| -------- | ----- | ----------- | -------------------- |
| password | `str` | `undefined` | The storage password |

### is_latest_address_unused(): bool

Determines whether all accounts have the latest address unused.

Returns _true_ if the latest address is unused.

### store_mnemonic(signer_type, mnemonic (optional)): bool

Stores a mnemonic for the given signer type.
If the mnemonic is not provided, we'll generate one.

| Param       | Type  | Default              | Description                                                      |
| ----------- | ----- | -------------------- | ---------------------------------------------------------------- |
| signer_type | `str` | `undefined`          | Should be _Stronghold_ , _LedgerNano_ , or _LedgerNanoSimulator_ |
| mnemonic    | `str` | `randomly generated` | The provided mnemonic or the randomly generated one              |

### generate_mnemonic(): str

Generates a new mnemonic.

Returns the generated mnemonic string.

### verify_mnemonic(mnemonic): void

Checks is the mnemonic is valid. If a mnemonic was generated with _generate_mnemonic()_ , the mnemonic here should match the generated.

| Param    | Type  | Default     | Description           |
| -------- | ----- | ----------- | --------------------- |
| mnemonic | `str` | `undefined` | The provided mnemonic |

### create_account(client_options): [AccountInitialiser](#accountinitialiser)

Creat a new account.

| Param          | Type                              | Default     | Description        |
| -------------- | --------------------------------- | ----------- | ------------------ |
| client_options | `[ClientOptions](#clientoptions)` | `undefined` | The client options |

Returns a constructed [AccountInitialiser](#accountinitialiser).

### remove_account(account_id): void

Deletes an account.

| Param      | Type  | Default     | Description                            |
| ---------- | ----- | ----------- | -------------------------------------- |
| account_id | `str` | `undefined` | The account with this id to be deleted |

### sync_accounts(): [AccountsSynchronizer](#accountssynchronizer)

Returns the [AccountsSynchronizer](#accountssynchronizer) to setup the process to synchronize the accounts with the Tangle.

### internal_transfer(from_account_id, to_account_id, amount): WalletMessage

Transfers an amount from an account to another.

| Param           | Type  | Default     | Description                                      |
| --------------- | ----- | ----------- | ------------------------------------------------ |
| from_account_id | `str` | `undefined` | The source of account id in the transfering      |
| to_account_id   | `str` | `undefined` | The destination of account id in the transfering |
| amount          | `int` | `undefined` | The transfer amount                              |

Returns the transfer's [WalletMessage](#walletmessage).

### backup(destination, Stronghold_password): str

Backups the storage to the given destination.

| Param               | Type  | Default     | Description                    |
| ------------------- | ----- | ----------- | ------------------------------ |
| destination         | `str` | `undefined` | The path to the backup file    |
| Stronghold_password | `str` | `undefined` | The backup Stronghold password |

Returns the full path to the backup file.

### import_accounts(source, Stronghold_password): void

Imports a database file.

| Param               | Type  | Default     | Description                    |
| ------------------- | ----- | ----------- | ------------------------------ |
| source              | `str` | `undefined` | The path to the backup file    |
| Stronghold_password | `str` | `undefined` | The backup Stronghold password |

### get_account(account_id): [AccountHandle](#accounthandle)

Gets the account with the given identifier or index.

| Param      | Type  | Default     | Description                                          |
| ---------- | ----- | ----------- | ---------------------------------------------------- |
| account_id | `str` | `undefined` | The account id, alias, index or one of its addresses |

Returns the associated AccountHandle object or undefined if the account wasn't found.

### get_accounts(): list[[AccountHandle](#accounthandle)]

Gets all stored accounts.

Returns a list of [AccountHandle](#accounthandle).

### retry(account_id, message_id): [WalletMessage](#walletmessage)

Retries (promotes or reattaches) the given message.

| Param      | Type  | Default     | Description                                          |
| ---------- | ----- | ----------- | ---------------------------------------------------- |
| account_id | `str` | `undefined` | The account id, alias, index or one of its addresses |
| message_id | `str` | `undefined` | The message's identifier                             |

Returns the retried [WalletMessage](#walletmessage).

### reattach(account_id, message_id): [WalletMessage](#walletmessage)

Reattach the given message.

| Param      | Type  | Default     | Description                                          |
| ---------- | ----- | ----------- | ---------------------------------------------------- |
| account_id | `str` | `undefined` | The account id, alias, index or one of its addresses |
| message_id | `str` | `undefined` | The message's identifier                             |

Returns the reattached [WalletMessage](#walletmessage).

### promote(account_id, message_id): [WalletMessage](#walletmessage)

Promote the given message.

| Param      | Type  | Default     | Description                                          |
| ---------- | ----- | ----------- | ---------------------------------------------------- |
| account_id | `str` | `undefined` | The account id, alias, index or one of its addresses |
| message_id | `str` | `undefined` | The message's identifier                             |

Returns the promoted [WalletMessage](#walletmessage).

### get_balance_change_events(count (optional), skip (optional), from_timestamp (optional))

Gets the persisted balance change events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| count          | `number` | `0`     | The number of events to return (`0` to return all)           |
| skip           | `number` | `0`     | The number of events to skip                                 |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { accountId: string, address: string, balanceChange: { spent: number, received: number } }

### get_balance_change_event_count(from_timestamp (optional))

Gets the number of persisted balance change events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### get_transaction_confirmation_events(count (optional), skip (optional), from_timestamp (optional))

Gets the persisted transaction confirmation change events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| count          | `number` | `0`     | The number of events to return (`0` to return all)           |
| skip           | `number` | `0`     | The number of events to skip                                 |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { accountId: string, message: Message, confirmed: boolean }

### get_transaction_confirmation_event_count(from_timestamp (optional))

Gets the number of persisted transaction confirmation change events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### get_new_transaction_events(count (optional), skip (optional), from_timestamp (optional))

Gets the persisted new transaction events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| count          | `number` | `0`     | The number of events to return (`0` to return all)           |
| skip           | `number` | `0`     | The number of events to skip                                 |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { accountId: string, message: Message }

### get_new_transaction_event_count(from_timestamp (optional))

Gets the number of persisted new transaction events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### get_reattachment_events(count (optional), skip (optional), from_timestamp (optional))

Gets the persisted transaction reattachment events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| count          | `number` | `0`     | The number of events to return (`0` to return all)           |
| skip           | `number` | `0`     | The number of events to skip                                 |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { accountId: string, message: Message }

### get_reattachment_event_count(from_timestamp (optional))

Gets the number of persisted transaction reattachment events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

### get_broadcast_events(count (optional), skip (optional), from_timestamp (optional))

Gets the persisted transaction broadcast events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| count          | `number` | `0`     | The number of events to return (`0` to return all)           |
| skip           | `number` | `0`     | The number of events to skip                                 |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

Event object: { accountId: string, message: Message }

### get_broadcast_event_count(from_timestamp (optional))

Gets the number of persisted transaction broadcast events.

| Param          | Type     | Default | Description                                                  |
| -------------- | -------- | ------- | ------------------------------------------------------------ |
| from_timestamp | `number` | `null`  | Filter events that were stored after the given UTC timestamp |

## AccountSynchronizer

### gap_limit(limit): void

Set the number of address indexes that are generated.

| Param | Type  | Default     | Description                                      |
| ----- | ----- | ----------- | ------------------------------------------------ |
| limit | `int` | `undefined` | The number of address indexes that are generated |

### skip_persistence(): void

Skip saving new messages and addresses on the account object.
The found [SyncedAccount](#syncedaccount) is returned on the _execute_ call but won't be persisted on the database.

### address_index(address_index): void

Set the initial address index to start syncing.

| Param         | Type  | Default     | Description                                |
| ------------- | ----- | ----------- | ------------------------------------------ |
| address_index | `int` | `undefined` | The initial address index to start syncing |

### execute(): [SyncedAccount](#syncedaccount)

Syncs account with the tangle.
The account syncing process ensures that the latest metadata (balance, transactions) associated with an account is fetched from the tangle and is stored locally.

## AccountsSynchronizer

### gap_limit(limit): void

Set the number of address indexes that are generated on each account.

| Param | Type  | Default     | Description                                                      |
| ----- | ----- | ----------- | ---------------------------------------------------------------- |
| limit | `int` | `undefined` | The number of address indexes that are generated on each account |

### address_index(address_index): void

Set the initial address index to start syncing on each account.

| Param         | Type  | Default     | Description                                                |
| ------------- | ----- | ----------- | ---------------------------------------------------------- |
| address_index | `int` | `undefined` | The initial address index to start syncing on each account |

### execute(): list[SyncedAccount](#syncedaccount)

Syncs the accounts with the tangle.

## Transfer

### constructor(amount, address, indexation (optional), remainder_value_strategy (optional): str, skip_sync (optional), output_kind (optional)): [Transfer](#transfer)

The _Transfer_ object used in [SyncedAccount](#syncedaccount)

| Param                    | Type                        | Default           | Description                                                         |
| ------------------------ | --------------------------- | ----------------- | ------------------------------------------------------------------- |
| amount                   | `int`                       | `undefined`       | The amount to transfer                                              |
| address                  | `str`                       | `undefined`       | The address to send                                                 |
| indexation               | `[Indexation](#indexation)` | `null`            | The indexation payload                                              |
| remainder_value_strategy | `str`                       | `_ChangeAddress_` | Should be _ReuseAddress_ or _ChangeAddress_                         |
| skip_sync                | `bool`                      | `False`           | Whether to skip the sync process                                    |
| output_kind              | `str`                       | `null`            | Should be _SignatureLockedSingle_ or _SignatureLockedDustAllowance_ |

## TransferOutput

| Param                    | Type                        | Default           | Description                                                         |
| ------------------------ | --------------------------- | ----------------- | ------------------------------------------------------------------- |
| amount                   | `int`                       | `undefined`       | The amount to transfer                                              |
| address                  | `str`                       | `undefined`       | The address to send                                                 |
| output_kind              | `str`                       | `null`            | Should be _SignatureLockedSingle_ or _SignatureLockedDustAllowance_ |

## TransferWithOutputs

### constructor(outputs: list[TransferOutput], indexation (optional), remainder_value_strategy (optional): str, skip_sync (optional)): [TransferWithOutputs](#transferwithoutputs)

The _Transfer_ object used in [SyncedAccount](#syncedaccount)

| Param                    | Type                        | Default           | Description                                                         |
| ------------------------ | --------------------------- | ----------------- | ------------------------------------------------------------------- |
| outputs                   | `list[TransferOutput]`     | `undefined`       | The amount to transfer                                              |
| indexation               | `[Indexation](#indexation)` | `null`            | The indexation payload                                              |
| remainder_value_strategy | `str`                       | `_ChangeAddress_` | Should be _ReuseAddress_ or _ChangeAddress_                         |
| skip_sync                | `bool`                      | `False`           | Whether to skip the sync process                                    |

## SyncedAccount

The result of a _sync_ operation on an Account.

### account_handle(): [AccountHandle](#accounthandle)

Get the [AccountHandle](#accounthandle) of this account

Returns the [AccountHandle](#accounthandle).

### deposit_address(): [Address](#address)

Get the deposit_address of this account.

Returns the [Address](#address).

### messages(): [WalletMessage](#walletmessage)

Get the messages of this account.

Returns the [WalletMessage](#walletmessage).

### addresses()

Get the addresses of this account.

Returns the list of [WalletMessage](#walletmessage).

## AccountHandle

### id(): str

Returns the account ID.

### signer_type(): str

Returns the signer type of this account.

### index(): int

Returns the account index.

### alias(): str

Returns the account alias.

### created_at(): int

Returns the created UNIX timestamp.

### last_synced_at(): int or None (it did not be synced before)

Returns the last synced UNIX timestamp.

### client_options(): [ClientOptions](#clientoptions)

Returns the client options of this account.

### bech32_hrp(): str

Returns the Bech32 HRP string.

### sync(): [AccountSynchronizer](#accountsynchronizer)

Returns the [AccountSynchronizer](#accountsynchronizer) to setup the process to synchronize this account with the Tangle.

### transfer(transfer_obj): [WalletMessage](#walletmessage)

Transfer tokens.

| Param        | Type                    | Default     | Description                  |
| ------------ | ----------------------- | ----------- | ---------------------------- |
| transfer_obj | `[Transfer](#transfer)` | `undefined` | The transfer we want to make |

Returns the [WalletMessage](#walletmessage) which makes the transfering.

### transfer_with_outputs(transfer_obj): [WalletMessage](#walletmessage)

Transfer tokens.

| Param        | Type                    | Default     | Description                  |
| ------------ | ----------------------- | ----------- | ---------------------------- |
| transfer_obj | `[Transfer](#transfer)` | `undefined` | The transfer we want to make |

Returns the [WalletMessage](#walletmessage) which makes the transfering.

### retry(message_id): [WalletMessage](#walletmessage)

Retries (promotes or reattaches) the given message.

| Param      | Type  | Default     | Description              |
| ---------- | ----- | ----------- | ------------------------ |
| message_id | `str` | `undefined` | The message's identifier |

Returns the retried [WalletMessage](#walletmessage).

### reattach(message_id): [WalletMessage](#walletmessage)

Reattach the given message.

| Param      | Type  | Default     | Description              |
| ---------- | ----- | ----------- | ------------------------ |
| message_id | `str` | `undefined` | The message's identifier |

Returns the reattached [WalletMessage](#walletmessage).

### promote(message_id): [WalletMessage](#walletmessage)

Promote the given message.

| Param      | Type  | Default     | Description              |
| ---------- | ----- | ----------- | ------------------------ |
| message_id | `str` | `undefined` | The message's identifier |

Returns the promoted [WalletMessage](#walletmessage).

### consolidate_outputs(): list[WalletMessage](#walletmessage)

Consolidates the account addresses outputs.

Returns the list of generated [WalletMessage](#walletmessage).

### generate_address(): list[[Address](#address)]

Returns a new unused address and links it to this account.

### get_unused_address(): [Address](#address)

Synchronizes the account addresses with the Tangle and returns the latest address in the account, which is an address without balance.

Returns the latest address in the account.

### is_latest_address_unused(): bool

Syncs the latest address with the Tangle and determines whether it's unused or not.
An unused address is an address without balance and associated message history.
Note that such address might have been used in the past, because the message history might have been pruned by the node.

Returns _true_ if the latest address in the account is unused.

### latest_address(): [Address](#address)

Returns the most recent address of the account.

### addresses(): list[[Address](#address)]

Returns a list of [Address](#address) in the account.

### balance(): [AccountBalance](#accountbalance)

Gets the account balance information.

Returns the [AccountBalance](#accountbalance) in this account.

### get_node_info(url (optional), auth (optional)): NodeInfoWrapper

Gets information about the node.

| Param | Type        | Default     | Description           |
| ----- | ----------- | ----------- | --------------------- |
| url   | `str`       | `undefined` | The node url          |
| auth  | `list[str]` | `undefined` | The node auth options |

Returns the [NodeInfoWrapper](#nodeinfowrapper)

### set_alias(alias): void

Updates the account alias.

| Param | Type  | Default     | Description              |
| ----- | ----- | ----------- | ------------------------ |
| alias | `str` | `undefined` | The account alias to set |

### set_client_options(options): void

Updates the account's client options.

| Param   | Type                              | Default     | Description               |
| ------- | --------------------------------- | ----------- | ------------------------- |
| options | `[ClientOptions](#clientoptions)` | `undefined` | The client options to set |

### message_count(message_type (optional)): int

Returns the number of messages associated with the account.

| Param        | Type  | Default     | Description                                                           |
| ------------ | ----- | ----------- | --------------------------------------------------------------------- |
| message_type | `str` | `undefined` | Should be _Received_ , _Sent_ , _Failed_ , _Unconfirmed_ , or _Value_ |

### list_messages(count, from, message_type (optional)): list([WalletMessage](#walletmessage))

Get the list of messages of this account.

| Param        | Type  | Default     | Description                                                           |
| ------------ | ----- | ----------- | --------------------------------------------------------------------- |
| count        | `int` | `undefined` | The count of the messages to get                                      |
| from         | `int` | `undefined` | The iniital address index                                             |
| message_type | `str` | `undefined` | Should be _Received_ , _Sent_ , _Failed_ , _Unconfirmed_ , or _Value_ |

### list_spent_addresses(): list[[Address](#address)]

Returns the list of spent [Address](#address) in the account.

### get_message(message_id): WalletMessage](#walletmessage) (optional)

Get the [WalletMessage](#walletmessage) by the message identifier in the account if it exists.

## AccountInitialiser

### signer_type(signer_type): void

Sets the account type.

| Param       | Type  | Default       | Description                                                      |
| ----------- | ----- | ------------- | ---------------------------------------------------------------- |
| signer_type | `str` | `signer_type` | Should be _Stronghold_ , _LedgerNano_ , or _LedgerNanoSimulator_ |

### alias(alias): void

Defines the account alias. If not defined, we'll generate one.

| Param | Type  | Default     | Description       |
| ----- | ----- | ----------- | ----------------- |
| alias | `str` | `undefined` | The account alias |

### created_at(created_at): void

Time of account creation.

| Param      | Type  | Default     | Description               |
| ---------- | ----- | ----------- | ------------------------- |
| created_at | `u64` | `undefined` | The account creation time |


### messages(messages): void

Messages associated with the seed.
The account can be initialised with locally stored messages.

| Param    | Type                                    | Default     | Description                 |
| -------- | --------------------------------------- | ----------- | --------------------------- |
| messages | `list([WalletMessage](#walletmessage))` | `undefined` | The locally stored messages |

### addresses(addresses): list([WalletAddress](#walletaddress))

Address history associated with the seed.
The account can be initialised with locally stored address history.

| Param     | Type                                    | Default     | Description              |
| --------- | --------------------------------------- | ----------- | ------------------------ |
| addresses | `list([WalletAddress](#walletaddress))` | `undefined` | The historical addresses |


### skip_persistence(): void

Skips storing the account to the database.

### initialise(): [AccountHandle](#accounthandle)

Initialises the account.

Returns the initilized [AccountHandle](#accounthandle)

## Event Listeners

### on_balance_change(callback): list[int]

Listen to balance changes.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_balance_change_listener(list[int]): void

Removes the balance change listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_new_transaction(callback): list[int]

Listen to new messages.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_new_transaction_listener(list[int]): void

Removes the new transaction listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_confirmation_state_change(callback): list[int]

Listen to transaction confirmation state change.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_confirmation_state_change_listener(list[int]): void

Removes the new transaction listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_reattachment(callback): list[int]

Listen to transaction reattachment.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_reattachment_listener(list[int]): void

Removes the reattachment listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_broadcast(callback): list[int]

Listen to transaction broadcast.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_broadcast_listener(list[int]): void

Removes the broadcast listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_error(callback): list[int]

Listen to errors.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_error_listener(list[int]): void

Removes the error listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_stronghold_status_change(callback): list[int]

Listen to Stronghold status change events.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_stronghold_status_change_listener(list[int]): void

Removes the Stronghold status change listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_transfer_progress(callback): list[int]

Listen to transfer progress events.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_transfer_progress_listener(list[int]): void

Removes the transfer progress listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

### on_migration_progress(callback): list[int]

Listen to migration progress events.

| Param      | Type       | Default     | Description           |
| ---------- | ---------- | ----------- | --------------------- |
| [callback] | `function` | `undefined` | The callback function |

Returns the event id as list[int].

### remove_migration_progress_listener(list[int]): void

Removes the migration progress listener associated with the given identifier.

| Param | Type        | Default     | Description  |
| ----- | ----------- | ----------- | ------------ |
| [id]  | `list[int]` | `undefined` | The event id |

## WalletAddress

A dict with the following key:value pairs.

```python
wallet_address = {
    'address': str,
    'balance': int,
    'key_index': int,
    'internal': bool,
    'outputs': dict[(string, WalletAddressOutput)],
}
```

Please refer to [WalletAddressOutput](#walletaddressoutput) for the details of this type.

## WalletAddressOutput

A dict with the following key:value pairs.

```python
wallet_address_output = {
    'transaction_id': str,
    'message_id': str,
    'index': int,
    'amount': int,
    'is_spent': bool,
    'address': str,
    'kind': str,
}
}
```

## Address

A dict with the following key:value pairs.

```python
address = {
    'address': AddressWrapper,
    'balance': int,
    'key_index': int,
    'internal': bool,
    'outputs': list[AddressOutput],
}
```

Please refer to [AddressWrapper](#addresswrapper) and [AddressOutput](#addressoutput) for the details of this type.

## AddressWrapper

A dict with the following key:value pairs.

```python
address_wrapper = {
    'inner': str
}
```

## AddressOutput

A dict with the following key:value pairs.

```python
address_output = {
    'transaction_id': str,
    'message_id': str,
    'index': int,
    'amount': int,
    'is_spent': bool,
    'address': AddressWrapper,
}
```

Please refer to [AddressWrapper](#addresswrapper) for the details of this type.

## AccountBalance

A dict with the following key:value pairs.

```python
account_balance = {
    'total': int,
    'available': int,
    'incoming': int,
    'outgoing': int,
}
```

## ClientOptions

A dict with the following key:value pairs.

```python
client_options = {
    'nodes': list[[Node](#node)] (optional),
    'primary_node': [Node](#node)] (optional), 
    'primary_pow_node': [Node](#node)] (optional),
    'node_pool_urls': list[str] (optional),
    'network': str (optional),
    'mqtt_enabled': bool (optional),
    'mqtt_broker_options': [BrokerOptions](#brokeroptions) (optional),
    'local_pow': bool (optional),
    'node_sync_interval': int (optional), # in milliseconds
    'node_sync_enabled': bool (optional),
    'request_timeout': int (optional), # in milliseconds
    'api_timeout': {
        'GetTips': int (optional) # in milliseconds
        'PostMessage': int (optional) # in milliseconds
        'GetOutput': int (optional) # in milliseconds
    } (optional)
}
```

Note that this message object in `wallet.rs` is not the same as the message object in `iota.rs`.

## Node

A dict with the following key:value pairs.

```python
node = {
    'url': string,
    'auth': NodeAuth (optional),
    'disabled': bool,
}
```

## NodeAuth

A dict with the following key:value pairs.

```python
node = {
    'username': string,
    'password': string,
}
```

## BrokerOptions

A dict with the following key:value pairs.

```python
broker_options = {
    'automatic_disconnect': bool (optional),
    'timeout': int (optional),
    'use_ws': bool (optional),
    'port': u16 (optional),
    'max_reconnection_attempts': u64 (optional),
}
```

## WalletMessage

A dict with the following key:value pairs.

```python
wallet_message = {
    'id': str,
    'version': u64,
    'parents': list[str],
    'payload_length': int,
    'payload': Payload,
    'timestamp': int,
    'nonce': int,
    'confirmed': bool (optional),
    'broadcasted': bool
}
```

Please refer to [Payload](#payload) for the details of this type.

## Payload

A dict with the following key:value pairs.

```python
payload = {
    'transaction': list[Transaction] (optional),
    'milestone': list[Milestone] (optional),
    'indexation': list[Indexation] (optional),
}
```

Please refer to [Transaction](#transaction), [Milestone](#milestone), and [Indexation](#indexation) for the details of these types.

## Transaction

A dict with the following key:value pairs.

```python
transaction = {
    'essence': {
        regular: RegularEssence
    },
    'unlock_blocks': list[UnlockBlock],
}
```

Please refer to [RegularEssence](#regularessence) and [UnlockBlock](#unlockblock) for the details of these types.

## Milestone

A dict with the following key:value pairs.

```python
milestone = {
    'essence': MilestonePayloadEssence,
    'signatures': list[bytes],
}
```

Please refer to [MilestonePayloadEssence](#milestonepayloadessence) for the details of this type.

## MilestonePayloadEssence

A dict with the following key:value pairs.

```python
milestone_payload_essence = {
    'index': int,
    'timestamp': int,
    'parents': list[str],
    'merkle_proof': bytes,
    'public_keys': bytes
}
```

## Indexation

A dict with the following key:value pairs.

```python
indexation = {
    'index': bytes,
    'data': bytes
}
```

## RegularEssenceEssence

A dict with the following key:value pairs.

```python
transaction_regular_essence = {
    'inputs': list[Input],
    'outputs': list[Output],
    'payload': Payload (optional),
    'internal': bool,
    'incoming': bool,
    'value': int,
    'remainder_value': int,
}
```
Please refer to [Input](#input), [Output](#output), and [Payload](#payload) for the details of these types.

## Output

A dict with the following key:value pairs.

```python
output = {
    'address': str,
    'amount': int
}
```

## Input

A dict with the following key:value pairs.

```python
input = {
    'transaction_id': str,
    'index': int
}
```

## UnlockBlock

A dict with the following key:value pairs.

```python
unlock_block = {
    'signature': Ed25519Signature (optional),
    'reference': int (optional)
}
```

Please refer to [Ed25519Signature](#ed25519signature) for the details of this type.

## Ed25519Signature

A dict with the following key:value pairs.

```python
ed25519_signature = {
    'public_key': bytes,
    'public_key': bytes
}
```
