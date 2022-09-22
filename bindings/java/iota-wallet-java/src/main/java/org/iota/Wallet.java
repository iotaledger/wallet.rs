// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.api.GsonSingleton;
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

    public AccountHandle createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        Account a = GsonSingleton.getInstance().fromJson(callBaseApi(new ClientCommand("CreateAccount", o)), Account.class);
        AccountHandle handle = new AccountHandle(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    public AccountHandle getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        Account a =  GsonSingleton.getInstance().fromJson(callBaseApi(new ClientCommand("GetAccount", GsonSingleton.getInstance().toJsonTree(accountIdentifier))), Account.class);
        AccountHandle handle = new AccountHandle(this, new AccountIndex(a.getIndex()));

        return handle;
    }

    public AccountHandle[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new ClientCommand("GetAccounts"));

        AccountHandle[] accountHandles = new AccountHandle[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountHandles[i] = new AccountHandle(this, new AccountIndex(GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), Account.class).getIndex()));

        return accountHandles;
    }


    public void backup(String destination, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("destination", destination);
        o.addProperty("password", password);

        callBaseApi(new ClientCommand("Backup", o));
    }

    public void changeStrongholdPassword(String currentPassword, String newPassword) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("currentPassword", currentPassword);
        o.addProperty("newPassword", newPassword);

        callBaseApi(new ClientCommand("ChangeStrongholdPassword", o));
    }

    public void clearStrongholdPassword() throws WalletException {
        callBaseApi(new ClientCommand("ClearStrongholdPassword"));
    }

    public boolean isStrongholdPasswordAvailable() throws WalletException {
        return callBaseApi(new ClientCommand("IsStrongholdPasswordAvailable")).getAsBoolean();
    }

    public void recoverAccounts(int accountStartIndex, int accountGapLimit, int addressGapLimit, SyncOptions syncOptions) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("accountStartIndex", accountStartIndex);
        o.addProperty("accountGapLimit", accountGapLimit);
        o.addProperty("addressGapLimit", addressGapLimit);
        o.add("syncOptions", GsonSingleton.getInstance().toJsonTree(syncOptions));

        callBaseApi(new ClientCommand("RecoverAccounts", o));
    }

    public void restoreBackup(String source, String password) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("source", source);
        o.addProperty("password", password);

        callBaseApi(new ClientCommand("RestoreBackup", o));
    }

    public void removeLatestAccount() throws WalletException {
        callBaseApi(new ClientCommand("RemoveLatestAccount"));
    }

    public String generateMnemonic() throws WalletException {
        return callBaseApi(new ClientCommand("GenerateMnemonic")).getAsString();
    }

    public String verifyMnemonic(String mnemonic) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(mnemonic);
        return callBaseApi(new ClientCommand("VerifyMnemonic", p)).getAsString();
    }

    public String setClientOptions(ClientConfig config) throws WalletException {
        return callBaseApi(new ClientCommand("SetClientOptions", GsonSingleton.getInstance().toJsonTree(config))).getAsString();
    }

    public LedgerNanoStatus getLedgerNanoStatus() throws WalletException {
        return GsonSingleton.getInstance().fromJson(callBaseApi(new ClientCommand("GetLedgerNanoStatus")), LedgerNanoStatus.class);
    }

    public JsonObject getNodeInfo(String url, NodeAuth auth) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("url", url);
        p.add("auth", GsonSingleton.getInstance().toJsonTree(auth));

        return (JsonObject) callBaseApi(new ClientCommand("GetNodeInfo", p));
    }

    public void setStrongholdPassword(String password) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(password);
        callBaseApi(new ClientCommand("SetStrongholdPassword", p));
    }

    public void setStrongholdPassword(int interval) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(interval);
        callBaseApi(new ClientCommand("SetStrongholdPasswordClearInterval", p));
    }

    public void storeMnemonic(String mnemonic) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(mnemonic);
        callBaseApi(new ClientCommand("StoreMnemonic", p));
    }

    public void startBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException {
        JsonObject o = new JsonObject();
        o.add("options", GsonSingleton.getInstance().toJsonTree(options));
        o.addProperty("intervalInMilliseconds", intervalInMilliseconds);

        callBaseApi(new ClientCommand("StartBackgroundSync", o));
    }

    public void stopBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException {
        callBaseApi(new ClientCommand("StopBackgroundSync"));
    }

    public void emitTestEvent(JsonElement event) throws WalletException {
        callBaseApi(new ClientCommand("EmitTestEvent", event));
    }

    public String bech32ToHex(String bech32) throws WalletException {
        return callBaseApi(new ClientCommand("Bech32ToHex", new JsonPrimitive(bech32))).getAsString();
    }

    public String hexToBech32(String hex, String bech32Hrp) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("hex", hex);
        p.addProperty("bech32Hrp", bech32Hrp);

        return callBaseApi(new ClientCommand("HexToBech32", p)).getAsString();
    }

}