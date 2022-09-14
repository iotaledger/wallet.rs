// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota;

import com.google.gson.*;
import org.iota.api.NativeApi;
import org.iota.types.*;
import org.iota.types.account_methods.*;
import org.iota.types.expections.WalletException;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;

public class Wallet extends NativeApi {

    public Wallet(WalletConfig config) {
        super(config);
    }

    public Account createAccount(String alias) throws WalletException {
        JsonObject o = new JsonObject();
        o.addProperty("alias", alias);

        return new Gson().fromJson(callBaseApi(new ClientCommand("CreateAccount", o)), Account.class);
    }

    public Account getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        JsonPrimitive p = null;

        if (accountIdentifier instanceof AccountIndex) {
            p = new JsonPrimitive(((AccountIndex) accountIdentifier).getAccountIndex());
        } else if (accountIdentifier instanceof AccountAlias) {
            p = new JsonPrimitive(((AccountAlias) accountIdentifier).getAccountAlias());
        } else throw new RuntimeException("unknown account identifier");

        return new Gson().fromJson(callBaseApi(new ClientCommand("GetAccount", p)), Account.class);
    }

    public Account[] getAccounts() throws WalletException {
        JsonArray responsePayload = (JsonArray) callBaseApi(new ClientCommand("GetAccounts"));

        Account[] accounts = new Account[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accounts[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), Account.class);

        return accounts;
    }

    // Account Method APIs

    private JsonElement callAccountMethod(AccountIdentifier accountIdentifier, AccountMethod accountMethod) throws WalletException {
        JsonPrimitive p;

        if (accountIdentifier instanceof AccountIndex) {
            p = new JsonPrimitive(((AccountIndex) accountIdentifier).getAccountIndex());
        } else if (accountIdentifier instanceof AccountAlias) {
            p = new JsonPrimitive(((AccountAlias) accountIdentifier).getAccountAlias());
        } else throw new RuntimeException("unknown account identifier");

        JsonObject method = new JsonObject();
        method.addProperty("name", accountMethod.getClass().getSimpleName());
        method.add("data", new Gson().toJsonTree(accountMethod));

        JsonObject o = new JsonObject();
        o.add("accountId", p);
        o.add("method", method);

        return callBaseApi(new ClientCommand("CallAccountMethod", o));
    }

    public Output buildAliasOutput(AccountIdentifier accountIdentifier, BuildAliasOutput buildAliasOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, buildAliasOutput);
        return new Output(responsePayload);
    }

    public Output buildBasicOutput(AccountIdentifier accountIdentifier, BuildBasicOutput buildBasicOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, buildBasicOutput);
        return new Output(responsePayload);
    }

    public Output buildFoundryOutput(AccountIdentifier accountIdentifier, BuildFoundryOutput buildFoundryOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, buildFoundryOutput);
        return new Output(responsePayload);
    }

    public Output buildNftOutput(AccountIdentifier accountIdentifier, BuildNftOutput buildNftOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, buildNftOutput);
        return new Output(responsePayload);
    }

    public TaggedDataPayload burnNativeToken(AccountIdentifier accountIdentifier, BurnNativeToken burnNativeToken) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, burnNativeToken);
        return new TaggedDataPayload(responsePayload);
    }

    public TaggedDataPayload burnNft(AccountIdentifier accountIdentifier, BurnNft burnNft) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, burnNft);
        return new TaggedDataPayload(responsePayload);
    }

    public TaggedDataPayload consolidateOutputs(AccountIdentifier accountIdentifier, ConsolidateOutputs consolidateOutputs) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, consolidateOutputs);
        return new TaggedDataPayload(responsePayload);
    }

    public TaggedDataPayload destroyAlias(AccountIdentifier accountIdentifier, DestroyAlias destroyAlias) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, destroyAlias);
        return new TaggedDataPayload(responsePayload);
    }

    public TaggedDataPayload destroyFoundry(AccountIdentifier accountIdentifier, DestroyFoundry destroyFoundry) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, destroyFoundry);
        return new TaggedDataPayload(responsePayload);
    }

    public AccountAddress[] generateAddresses(AccountIdentifier accountIdentifier, GenerateAddresses generateAddresses) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, generateAddresses);

        AccountAddress[] accountAddress = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountAddress[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return accountAddress;
    }

    public Output getOutput(AccountIdentifier accountIdentifier, GetOutput getOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, getOutput);
        return new Output(responsePayload);
    }

    public Output getFoundryOutput(AccountIdentifier accountIdentifier, GetFoundryOutput getFoundryOutput) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, getFoundryOutput);
        return new Output(responsePayload);
    }

    public Output[] getOutputsWithAdditionalUnlockConditions(AccountIdentifier accountIdentifier, GetOutputsWithAdditionalUnlockConditions getOutputsWithAdditionalUnlockConditions) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, getOutputsWithAdditionalUnlockConditions);

        Output[] outputs = new Output[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputs[i] = new Output(responsePayload.get(i).getAsJsonObject());

        return outputs;
    }

    public TransactionPayload getTransaction(AccountIdentifier accountIdentifier, GetTransaction getTransaction) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, getTransaction);

        return new TransactionPayload(responsePayload);
    }

    public JsonElement getIncomingTransactionData(AccountIdentifier accountIdentifier, GetIncomingTransactionData getIncomingTransactionData) throws WalletException {
        return callAccountMethod(accountIdentifier, getIncomingTransactionData);
    }

    public AccountAddress[] listAddresses(AccountIdentifier accountIdentifier, ListAddresses method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public AccountAddress[] listAddressesWithUnspentOutputs(AccountIdentifier accountIdentifier, ListAddressesWithUnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public OutputData[] listOutputs(AccountIdentifier accountIdentifier, ListOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public TransactionPayload[] listPendingTransactions(AccountIdentifier accountIdentifier, ListPendingTransactions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        TransactionPayload[] transactionPayloads = new TransactionPayload[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactionPayloads[i] = new TransactionPayload(responsePayload.get(i).getAsJsonObject());

        return transactionPayloads;
    }

    public TransactionPayload[] listTransactions(AccountIdentifier accountIdentifier, ListTransactions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        TransactionPayload[] transactionPayloads = new TransactionPayload[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactionPayloads[i] = new TransactionPayload(responsePayload.get(i).getAsJsonObject());

        return transactionPayloads;
    }

    public OutputData[] listUnspentOutputs(AccountIdentifier accountIdentifier, ListUnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = new Gson().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public TaggedDataPayload meltNativeToken(AccountIdentifier accountIdentifier, MeltNativeToken method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TaggedDataPayload(responsePayload);
    }

    public String minimumRequiredStorageDeposit(AccountIdentifier accountIdentifier, MinimumRequiredStorageDeposit method) throws WalletException {
        return callAccountMethod(accountIdentifier, method).getAsJsonPrimitive().getAsString();
    }

    public TransactionPayload mintNfts(AccountIdentifier accountIdentifier, MintNfts method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public AccountBalance getBalance(AccountIdentifier accountIdentifier, GetBalance method) throws WalletException {
        return new Gson().fromJson(callAccountMethod(accountIdentifier, method), AccountBalance.class);
    }

    public Output prepareOutput(AccountIdentifier accountIdentifier, PrepareOutput method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new Output(responsePayload);
    }

    public PreparedTransactionData prepareTransaction(AccountIdentifier accountIdentifier, PrepareTransaction method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new PreparedTransactionData(responsePayload);
    }

    public PreparedTransactionData prepareSendAmount(AccountIdentifier accountIdentifier, PrepareSendAmount method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new PreparedTransactionData(responsePayload);
    }

    public AccountBalance syncAccount(AccountIdentifier accountIdentifier, SyncAccount method) throws WalletException {
        return new Gson().fromJson(callAccountMethod(accountIdentifier, method), AccountBalance.class);
    }

    public TransactionPayload sendAmount(AccountIdentifier accountIdentifier, SendAmount method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload sendMicroTransaction(AccountIdentifier accountIdentifier, SendMicroTransaction method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload sendNativeTokens(AccountIdentifier accountIdentifier, SendNativeTokens method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload sendNft(AccountIdentifier accountIdentifier, SendNft method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public void setAlias(AccountIdentifier accountIdentifier, SetAlias method) throws WalletException {
        callAccountMethod(accountIdentifier, method);
    }

    public TransactionPayload sendOutputs(AccountIdentifier accountIdentifier, SendOutputs method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload signTransactionEssence(AccountIdentifier accountIdentifier, SignTransactionEssence method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload submitAndStoreTransaction(AccountIdentifier accountIdentifier, SubmitAndStoreTransaction method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload tryClaimOutputs(AccountIdentifier accountIdentifier, TryClaimOutputs method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload claimOutputs(AccountIdentifier accountIdentifier, ClaimOutputs method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
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
        o.add("sync_options", new Gson().toJsonTree(sync_options));

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
        o.add("options", new Gson().toJsonTree(options));
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