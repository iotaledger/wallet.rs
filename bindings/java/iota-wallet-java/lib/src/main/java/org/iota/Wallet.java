// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.api.WalletCommand;
import org.iota.api.CustomGson;
import org.iota.api.NativeApi;
import org.iota.types.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;

public class Wallet extends NativeApi {

    public Wallet(WalletConfig config) {
        super(config);
    }

    // Account manager APIs

    /**
     * Create an account with the given alias and return an AccountHandle for it.
     *
     * @param alias The name of the account.
     * @return An AccountHandle object.
     */
    public AccountHandle createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        Account a = CustomGson.get().fromJson(callBaseApi(new WalletCommand("createAccount", o)), Account.class);
        AccountHandle handle = new AccountHandle(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    /**
     * Return a given account from the wallet.
     *
     * @param accountIdentifier The account identifier.
     * @return An AccountHandle object.
     */
    public AccountHandle getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        Account a =  CustomGson.get().fromJson(callBaseApi(new WalletCommand("getAccount", CustomGson.get().toJsonTree(accountIdentifier))), Account.class);
        AccountHandle handle = new AccountHandle(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    /**
     * Returns all the accounts from the wallet.
     *
     * @return An array of AccountHandles.
     */
    public AccountHandle[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new WalletCommand("getAccounts"));

        AccountHandle[] accountHandles = new AccountHandle[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountHandles[i] = new AccountHandle(this, new AccountIndex(CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Account.class).getIndex()));

        return accountHandles;
    }

    /**
     * Backup the wallet to the specified destination, encrypting it with the specified password.
     *
     * @param destination The path to the file to be created.
     * @param password The password to encrypt the backup with.
     */
    public void backup(String destination, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("destination", destination);
        o.addProperty("password", password);

        callBaseApi(new WalletCommand("backup", o));
    }

    /**
     * Change the password of the Stronghold file.
     *
     * @param currentPassword The current password for the Stronghold
     * @param newPassword The new password you want to use for your Stronghold.
     */
    public void changeStrongholdPassword(String currentPassword, String newPassword) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("currentPassword", currentPassword);
        o.addProperty("newPassword", newPassword);

        callBaseApi(new WalletCommand("changeStrongholdPassword", o));
    }

    /**
     * Clears the Stronghold password from memory.
     */
    public void clearStrongholdPassword() throws WalletException {
        callBaseApi(new WalletCommand("clearStrongholdPassword"));
    }

    /**
     * Checks if the Stronghold password is available.
     *
     * @return A boolean value.
     */
    public boolean isStrongholdPasswordAvailable() throws WalletException {
        return callBaseApi(new WalletCommand("isStrongholdPasswordAvailable")).getAsBoolean();
    }

    /**
     * Find accounts with unspent outputs.
     */
    public void recoverAccounts(int accountStartIndex, int accountGapLimit, int addressGapLimit, SyncOptions syncOptions) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("accountStartIndex", accountStartIndex);
        o.addProperty("accountGapLimit", accountGapLimit);
        o.addProperty("addressGapLimit", addressGapLimit);
        o.add("syncOptions", CustomGson.get().toJsonTree(syncOptions));

        callBaseApi(new WalletCommand("recoverAccounts", o));
    }

    /**
     * Restore a backup from a Stronghold file
     * Replaces client_options, coin_type, secret_manager and accounts. Returns an error if accounts were already
     * created If Stronghold is used as secret_manager, the existing Stronghold file will be overwritten. If a
     * mnemonic was stored, it will be gone.
     *
     * @param source The path to the backup file.
     * @param password The password you used to encrypt the backup file.
     */
    public void restoreBackup(String source, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("source", source);
        o.addProperty("password", password);

        callBaseApi(new WalletCommand("restoreBackup", o));
    }

    /**
     * Removes the latest account (account with the largest account index).
     */
    public void removeLatestAccount() throws WalletException {
        callBaseApi(new WalletCommand("removeLatestAccount"));
    }

    /**
     * Generate a mnemonic phrase
     *
     * @return A string of words.
     */
    public String generateMnemonic() throws WalletException {
        return callBaseApi(new WalletCommand("generateMnemonic")).getAsString();
    }

    /**
     * Checks if the given mnemonic is valid.
     *
     * @param mnemonic The mnemonic to verify.
     */
    public void verifyMnemonic(String mnemonic) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(mnemonic);
        callBaseApi(new WalletCommand("verifyMnemonic", p));
    }

    /**
     * Updates the client options for all accounts.
     *
     * @param config A ClientConfig object that contains the options to set.
     */
    public void setClientOptions(ClientConfig config) throws WalletException {
        callBaseApi(new WalletCommand("setClientOptions", CustomGson.get().toJsonTree(config)));
    }

    /**
     * Get the status of the Ledger Nano.
     *
     * @return The status of the Ledger Nano
     */
    public LedgerNanoStatus getLedgerNanoStatus() throws WalletException {
        return CustomGson.get().fromJson(callBaseApi(new WalletCommand("getLedgerNanoStatus")), LedgerNanoStatus.class);
    }

    /**
     * Get node information.
     *
     * @param url The URL of the node you want information from.
     * @param auth The authentication information for the node.
     * @return A JsonObject
     */
    public JsonObject getNodeInfo(String url, NodeAuth auth) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("url", url);
        p.add("auth", CustomGson.get().toJsonTree(auth));

        return (JsonObject) callBaseApi(new WalletCommand("getNodeInfo", p));
    }

    /**
     * Set the stronghold password clear interval.
     *
     * @param password The password to set for the stronghold.
     */
    public void setStrongholdPassword(String password) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(password);
        callBaseApi(new WalletCommand("setStrongholdPassword", p));
    }

    /**
     * Set the stronghold password clear interval.
     *
     * @param interval The number of seconds to wait before clearing the password.
     */
    public void setStrongholdPasswordClearInterval(int interval) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(interval);
        callBaseApi(new WalletCommand("setStrongholdPasswordClearInterval", p));
    }

    /**
     * Store a mnemonic into the Stronghold vault.
     *
     * @param mnemonic The mnemonic to store.
     */
    public void storeMnemonic(String mnemonic) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(mnemonic);
        callBaseApi(new WalletCommand("storeMnemonic", p));
    }

    /**
     * Start a background sync with the specified options and interval.
     *
     * @param options The options for the sync.
     * @param intervalInMilliseconds The interval in milliseconds at which the background sync will be performed.
     */
    public void startBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException {
        JsonObject o = new JsonObject();
        o.add("options", CustomGson.get().toJsonTree(options));
        o.addProperty("intervalInMilliseconds", intervalInMilliseconds);

        callBaseApi(new WalletCommand("startBackgroundSync", o));
    }


    /**
     * Stop the background sync process.
     */
    public void stopBackgroundSync() throws WalletException {
        callBaseApi(new WalletCommand("stopBackgroundSync"));
    }

    /**
     * Emits an event for testing if the event system is working
     *
     * @param event The event to emit.
     */
    public void emitTestEvent(JsonElement event) throws WalletException {
        callBaseApi(new WalletCommand("emitTestEvent", event));
    }

    /**
     * Converts a bech32 address to a hex address.
     *
     * @param bech32 The bech32 string to convert to hex.
     * @return A hex string.
     */
    public String bech32ToHex(String bech32) throws WalletException {
        return callBaseApi(new WalletCommand("bech32ToHex", new JsonPrimitive(bech32))).getAsString();
    }

    /**
     * Converts a hex address to a bech32 address.
     *
     * @param hex The hex address to convert.
     * @param bech32Hrp The bech32 human-readable part.
     * @return The bech32 address.
     */
    public String hexToBech32(String hex, String bech32Hrp) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("hex", hex);
        p.addProperty("bech32Hrp", bech32Hrp);

        return callBaseApi(new WalletCommand("hexToBech32", p)).getAsString();
    }

}