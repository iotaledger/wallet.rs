package org.iota.apis;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.types.Account;
import org.iota.types.WalletConfig;
import org.iota.types.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;

public class WalletApi extends BaseApi {

    public WalletApi(WalletConfig walletConfig) {
        super(walletConfig);
    }

    public Account createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        JsonObject responsePayload = (JsonObject) callBaseApi(new ClientCommand("CreateAccount", o));

        return new Account(responsePayload);
    }

    public void removeLatestAccount() throws WalletException {
         callBaseApi(new ClientCommand("RemoveLatestAccount"));
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

}