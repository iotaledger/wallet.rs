package org.iota.apis;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.types.*;
import org.iota.types.expections.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;

public class WalletApi extends BaseApi {

    private AccountMethodApi accountMethodApi;

    public WalletApi(WalletConfig walletConfig) {
        super(walletConfig);
        accountMethodApi = new AccountMethodApi(walletConfig);
    }

    public Account createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        JsonObject responsePayload = (JsonObject) callBaseApi(new ClientCommand("CreateAccount", o));

        return new Account(responsePayload);
    }

    public Account getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        JsonPrimitive p = null;

        if (accountIdentifier instanceof AccountIndex) {
            p = new JsonPrimitive(((AccountIndex) accountIdentifier).getAccountIndex());
        } else if (accountIdentifier instanceof AccountAlias) {
            p = new JsonPrimitive(((AccountAlias) accountIdentifier).getAccountAlias());
        } else throw new RuntimeException("unknown account identifier");

        JsonObject responsePayload = (JsonObject) callBaseApi(new ClientCommand("GetAccount", p));

        return new Account(responsePayload);
    }

    public Account[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new ClientCommand("GetAccounts"));

        Account[] accounts = new Account[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accounts[i] = new Account(responsePayload.get(i).getAsJsonObject());

        return accounts;
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

    public void recoverAccounts(int accountGapLimit, int addressGapLimit, SyncOptions sync_options) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("accountGapLimit", accountGapLimit);
        o.addProperty("addressGapLimit", addressGapLimit);
        o.add("sync_options", sync_options.toJson());

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

    public void deleteAccountsAndDatabase() throws WalletException {
        callBaseApi(new ClientCommand("DeleteAccountsAndDatabase"));
    }

    public String generateMnemonic() throws WalletException {
        return callBaseApi(new ClientCommand("GenerateMnemonic")).getAsString();
    }

    public String verifyMnemonic(String mnemonic) throws WalletException {
        JsonPrimitive p = new JsonPrimitive(mnemonic);
        return callBaseApi(new ClientCommand("VerifyMnemonic", p)).getAsString();
    }

    public String setClientOptions(WalletConfig.ClientConfig config) throws WalletException {
        return callBaseApi(new ClientCommand("SetClientOptions", config.getJson())).getAsString();
    }

    public LedgerNanoStatus getLedgerNanoStatus() throws WalletException {
        JsonObject o = (JsonObject) callBaseApi(new ClientCommand("GetLedgerNanoStatus"));
        return new LedgerNanoStatus(o);
    }

    public JsonObject getNodeInfo(String url, NodeAuth auth) throws WalletException {
        JsonObject p = new JsonObject();
        p.addProperty("url", url);
        p.add("auth", auth.toJson());

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
        o.add("options", options.toJson());
        o.addProperty("intervalInMilliseconds", intervalInMilliseconds);

        callBaseApi(new ClientCommand("StartBackgroundSync", o));
    }

    public void stopBackgroundSync(SyncOptions options, int intervalInMilliseconds) throws WalletException {
        callBaseApi(new ClientCommand("StopBackgroundSync"));
    }

    public void emitTestEvent(WalletEvent event) throws WalletException {
        callBaseApi(new ClientCommand("EmitTestEvent", event.toJson()));
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