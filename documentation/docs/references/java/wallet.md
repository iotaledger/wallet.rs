---
description: Official IOTA Wallet Library Software Rust API reference.
image: /img/logo/wallet_light.png
keywords:
- api
- rust
- cargo
- crate
---
# Wallet API Reference

## `public AccountHandle createAccount(String alias) throws WalletException`

Create an account with the given alias and return an AccountHandle for it.

* **Parameters:** `alias` — The name of the account.
* **Returns:** An AccountHandle object.

## `public AccountHandle getAccount(AccountIdentifier accountIdentifier) throws WalletException`

Return a given account from the wallet.

* **Parameters:** `accountIdentifier` — The account identifier.
* **Returns:** An AccountHandle object.

## `public AccountHandle[] getAccounts() throws WalletException`

Returns all the accounts from the wallet.

* **Returns:** An array of AccountHandles.

## `public void backup(String destination, String password) throws WalletException`

Backup the wallet to the specified destination, encrypting it with the specified password.

* **Parameters:**
    * `destination` — The path to the file to be created.
    * `password` — The password to encrypt the backup with.

## `public void changeStrongholdPassword(String currentPassword, String newPassword) throws WalletException`

Change the password of the Stronghold file.

* **Parameters:**
    * `currentPassword` — The current password for the Stronghold
    * `newPassword` — The new password you want to use for your Stronghold.

## `public void clearStrongholdPassword() throws WalletException`

Clears the Stronghold password from memory.

## `public boolean isStrongholdPasswordAvailable() throws WalletException`

Checks if the Stronghold password is available.

* **Returns:** A boolean value.

## `public void recoverAccounts(int accountStartIndex, int accountGapLimit, int addressGapLimit, SyncOptions syncOptions) throws WalletException`

Find accounts with unspent outputs.

## `public void restoreBackup(String source, String password) throws WalletException`

Restore a backup from a Stronghold file Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already created If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a mnemonic was stored, it will be gone.

* **Parameters:**
    * `source` — The path to the backup file.
    * `password` — The password you used to encrypt the backup file.

## `public void removeLatestAccount() throws WalletException`

Removes the latest account (account with the largest account index).

## `public String generateMnemonic() throws WalletException`

Generate a mnemonic phrase

* **Returns:** A string of words.

## `public void verifyMnemonic(String mnemonic) throws WalletException`

Checks if the given mnemonic is valid.

* **Parameters:** `mnemonic` — The mnemonic to verify.
* **Returns:** The wallet address.

## `public void setClientOptions(ClientConfig config) throws WalletException`

Updates the client options for all accounts.

* **Parameters:** `config` — A ClientConfig object that contains the options to set.

## `public LedgerNanoStatus getLedgerNanoStatus() throws WalletException`

Get the status of the Ledger Nano.

* **Returns:** The status of the Ledger Nano

## `public JsonObject getNodeInfo(String url, NodeAuth auth) throws WalletException`

Get node information.

* **Parameters:**
    * `url` — The URL of the node you want information from.
    * `auth` — The authentication information for the node.
* **Returns:** A JsonObject

## `public void setStrongholdPassword(String password) throws WalletException`

Set the stronghold password clear interval.

* **Parameters:** `password` — The password to set for the stronghold.

## `public void setStrongholdPasswordClearInterval(int interval) throws WalletException`

Set the stronghold password clear interval.

* **Parameters:** `interval` — The number of seconds to wait before clearing the password.

## `public void storeMnemonic(String mnemonic) throws WalletException`

Store a mnemonic into the Stronghold vault.

* **Parameters:** `mnemonic` — The mnemonic to store.

## `public void startBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException`

Start a background sync with the specified options and interval.

* **Parameters:**
    * `options` — The options for the sync.
    * `intervalInMilliseconds` — The interval in milliseconds at which the background sync will be performed.

## `public void stopBackgroundSync() throws WalletException`

Stop the background sync process.

## `public void emitTestEvent(JsonElement event) throws WalletException`

Emits an event for testing if the event system is working

* **Parameters:** `event` — The event to emit.

## `public String bech32ToHex(String bech32) throws WalletException`

Converts a bech32 address to a hex address.

* **Parameters:** `bech32` — The bech32 string to convert to hex.
* **Returns:** A hex string.

## `public String hexToBech32(String hex, String bech32Hrp) throws WalletException`

Converts a hex address to a bech32 address.

* **Parameters:**
    * `hex` — The hex address to convert.
    * `bech32Hrp` — The bech32 human-readable part.
* **Returns:** The bech32 address.