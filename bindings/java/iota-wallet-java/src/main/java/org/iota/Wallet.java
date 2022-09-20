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
import org.iota.types.account_methods.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.outputs.Output;
import org.iota.types.payload.TaggedDataPayload;
import org.iota.types.payload.TransactionPayload;

public class Wallet extends NativeApi {

    public Wallet(WalletConfig config) {
        super(config);
    }

    // Account manager APIs

    public Account createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        return GsonSingleton.getInstance().fromJson(callBaseApi(new ClientCommand("CreateAccount", o)), Account.class);
    }

    public Account getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callBaseApi(new ClientCommand("GetAccount", GsonSingleton.getInstance().toJsonTree(accountIdentifier))), Account.class);
    }

    public Account[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new ClientCommand("GetAccounts"));

        Account[] accounts = new Account[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accounts[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), Account.class);

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
        o.add("sync_options", GsonSingleton.getInstance().toJsonTree(sync_options));

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

    // Account Method APIs

    private JsonElement callAccountMethod(AccountIdentifier accountIdentifier, AccountMethod accountMethod) throws WalletException {
        JsonObject method = new JsonObject();
        method.addProperty("name", accountMethod.getClass().getSimpleName());
        JsonElement data = GsonSingleton.getInstance().toJsonTree(accountMethod);
        if(data.toString().equals("{}"))
            method.add("data", null);
        else
            method.add("data", data);

        JsonObject o = new JsonObject();
        o.add("accountId", GsonSingleton.getInstance().toJsonTree(accountIdentifier));
        o.add("method", method);

        return callBaseApi(new ClientCommand("CallAccountMethod", o));
    }

    public Output buildAliasOutput(AccountIdentifier accountIdentifier, BuildAliasOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public Output buildBasicOutput(AccountIdentifier accountIdentifier, BuildBasicOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public Output buildFoundryOutput(AccountIdentifier accountIdentifier, BuildFoundryOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public Output buildNftOutput(AccountIdentifier accountIdentifier, BuildNftOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public TaggedDataPayload burnNativeToken(AccountIdentifier accountIdentifier, BurnNativeToken method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }

    public TaggedDataPayload burnNft(AccountIdentifier accountIdentifier, BurnNft method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }


    public TaggedDataPayload consolidateOutputs(AccountIdentifier accountIdentifier, ConsolidateOutputs method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }


    public TaggedDataPayload destroyAlias(AccountIdentifier accountIdentifier, DestroyAlias method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }


    public TaggedDataPayload destroyFoundry(AccountIdentifier accountIdentifier, DestroyFoundry method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }

    public AccountAddress[] generateAddresses(AccountIdentifier accountIdentifier, GenerateAddresses method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] accountAddress = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountAddress[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return accountAddress;
    }

    public Output getOutput(AccountIdentifier accountIdentifier, GetOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public Output getFoundryOutput(AccountIdentifier accountIdentifier, GetFoundryOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public Output[] getOutputsWithAdditionalUnlockConditions(AccountIdentifier accountIdentifier, GetOutputsWithAdditionalUnlockConditions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        Output[] outputs = new Output[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputs[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), Output.class);

        return outputs;
    }

    public TransactionPayload getTransaction(AccountIdentifier accountIdentifier, GetTransaction method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public JsonElement getIncomingTransactionData(AccountIdentifier accountIdentifier, GetIncomingTransactionData method) throws WalletException {
        return callAccountMethod(accountIdentifier, method);
    }

    public AccountAddress[] listAddresses(AccountIdentifier accountIdentifier, ListAddresses method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public AccountAddress[] listAddressesWithUnspentOutputs(AccountIdentifier accountIdentifier, ListAddressesWithUnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public OutputData[] listOutputs(AccountIdentifier accountIdentifier, ListOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public TransactionPayload[] listPendingTransactions(AccountIdentifier accountIdentifier, ListPendingTransactions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        TransactionPayload[] transactionPayloads = new TransactionPayload[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactionPayloads[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), TransactionPayload.class);

        return transactionPayloads;
    }

    public TransactionPayload[] listTransactions(AccountIdentifier accountIdentifier, ListTransactions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        TransactionPayload[] transactionPayloads = new TransactionPayload[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactionPayloads[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), TransactionPayload.class);

        return transactionPayloads;
    }

    public OutputData[] listUnspentOutputs(AccountIdentifier accountIdentifier, ListUnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = GsonSingleton.getInstance().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public TaggedDataPayload meltNativeToken(AccountIdentifier accountIdentifier, DecreaseNativeTokenSupply method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TaggedDataPayload.class);
    }

    public String minimumRequiredStorageDeposit(AccountIdentifier accountIdentifier, MinimumRequiredStorageDeposit method) throws WalletException {
        return callAccountMethod(accountIdentifier, method).getAsJsonPrimitive().getAsString();
    }

    public TransactionPayload mintNfts(AccountIdentifier accountIdentifier, MintNfts method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public AccountBalance getBalance(AccountIdentifier accountIdentifier, GetBalance method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), AccountBalance.class);
    }

    public Output prepareOutput(AccountIdentifier accountIdentifier, PrepareOutput method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), Output.class);
    }

    public PreparedTransactionData prepareTransaction(AccountIdentifier accountIdentifier, PrepareTransaction method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), PreparedTransactionData.class);
    }

    public PreparedTransactionData prepareSendAmount(AccountIdentifier accountIdentifier, PrepareSendAmount method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), PreparedTransactionData.class);
    }

    public AccountBalance syncAccount(AccountIdentifier accountIdentifier, SyncAccount method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), AccountBalance.class);
    }

    public TransactionPayload sendAmount(AccountIdentifier accountIdentifier, SendAmount method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload sendMicroTransaction(AccountIdentifier accountIdentifier, SendMicroTransaction method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload sendNativeTokens(AccountIdentifier accountIdentifier, SendNativeTokens method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload sendNft(AccountIdentifier accountIdentifier, SendNft method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public void setAlias(AccountIdentifier accountIdentifier, SetAlias method) throws WalletException {
        callAccountMethod(accountIdentifier, method);
    }

    public TransactionPayload sendOutputs(AccountIdentifier accountIdentifier, SendOutputs method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload signTransactionEssence(AccountIdentifier accountIdentifier, SignTransactionEssence method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload submitAndStoreTransaction(AccountIdentifier accountIdentifier, SubmitAndStoreTransaction method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

    public TransactionPayload claimOutputs(AccountIdentifier accountIdentifier, ClaimOutputs method) throws WalletException {
        return GsonSingleton.getInstance().fromJson(callAccountMethod(accountIdentifier, method), TransactionPayload.class);
    }

}