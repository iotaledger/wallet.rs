// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.Wallet;
import org.iota.api.CustomGson;
import org.iota.api.NativeApi;
import org.iota.types.account_methods.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.OutputId;
import org.iota.types.ids.TransactionId;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.outputs.Output;
import org.iota.types.payload.TaggedDataPayload;
import org.iota.types.payload.TransactionPayload;

import java.lang.reflect.Type;
import java.util.Map;
import java.util.Set;

import static org.iota.api.NativeApi.callBaseApi;
@JsonAdapter(AccountHandleAdapter.class)
public class AccountHandle extends AbstractObject {

    private Wallet wallet;
    private AccountIdentifier accountIdentifier;

    public AccountHandle(Wallet wallet, AccountIdentifier accountIdentifier) {
        this.wallet = wallet;
        this.accountIdentifier = accountIdentifier;
    }

    public int getIndex() throws WalletException {
        return getAccountCopy().getIndex();
    }

    public int getCoinType() throws WalletException {
        return getAccountCopy().getCoinType();
    }

    public String getAlias() throws WalletException {
        return getAccountCopy().getAlias();
    }

    public AccountAddress[] getPublicAddresses() throws WalletException {
        return getAccountCopy().getPublicAddresses();
    }

    public AccountAddress[] getInternalAddresses() throws WalletException {
        return getAccountCopy().getInternalAddresses();
    }

    public AddressWithUnspentOutputs[] getAddressesWithUnspentOutputs() throws WalletException {
        return getAccountCopy().getAddressesWithUnspentOutputs();
    }

    public Map<OutputId, OutputData> getOutputs() throws WalletException {
        return getAccountCopy().getOutputs();
    }

    public Set<OutputId> getLockedOutputs() throws WalletException {
        return getAccountCopy().getLockedOutputs();
    }

    public Map<OutputId, OutputData> getUnspentOutputs() throws WalletException {
        return getAccountCopy().getUnspentOutputs();
    }

    public Map<TransactionId, Transaction> getTransactions() throws WalletException {
        return getAccountCopy().getTransactions();
    }

    public Set<TransactionId> getPendingTransactions() throws WalletException {
        return getAccountCopy().getPendingTransactions();
    }

    public Map<TransactionId, Map.Entry<TransactionPayload, OutputResponse[]>> getIncomingTransactions() throws WalletException {
        return getAccountCopy().getIncomingTransactions();
    }

    public Account getAccountCopy() throws WalletException {
        return CustomGson.get().fromJson(callBaseApi(new NativeApi.ClientCommand("GetAccount", CustomGson.get().toJsonTree(accountIdentifier))), Account.class);
    }

    // Account Method APIs

    private JsonElement callAccountMethod(AccountMethod accountMethod) throws WalletException {
        JsonObject method = new JsonObject();
        method.addProperty("name", accountMethod.getClass().getSimpleName());
        JsonElement data = CustomGson.get().toJsonTree(accountMethod);
        if(data.toString().equals("{}"))
            method.add("data", null);
        else
            method.add("data", data);

        JsonObject o = new JsonObject();
        o.add("accountId", CustomGson.get().toJsonTree(accountIdentifier));
        o.add("method", method);

        return callBaseApi(new NativeApi.ClientCommand("CallAccountMethod", o));
    }

    public Output buildAliasOutput(BuildAliasOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public Output buildBasicOutput(BuildBasicOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public Output buildFoundryOutput(BuildFoundryOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public Output buildNftOutput(BuildNftOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public Transaction burnNativeToken(BurnNativeToken method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction burnNft(BurnNft method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }


    public Transaction consolidateOutputs(ConsolidateOutputs method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }


    public Transaction destroyAlias(DestroyAlias method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }


    public Transaction destroyFoundry(DestroyFoundry method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public AccountAddress[] generateAddresses(GenerateAddresses method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(method);

        AccountAddress[] accountAddress = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountAddress[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return accountAddress;
    }

    public OutputData getOutput(GetOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), OutputData.class);
    }

    public Output getFoundryOutput(GetFoundryOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public Output[] getOutputsWithAdditionalUnlockConditions(GetOutputsWithAdditionalUnlockConditions method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(method);

        Output[] outputs = new Output[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputs[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Output.class);

        return outputs;
    }

    public Transaction getTransaction(GetTransaction method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public JsonElement getIncomingTransactionData(GetIncomingTransactionData method) throws WalletException {
        return callAccountMethod(method);
    }

    public AccountAddress[] listAddresses() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new Addresses());

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public AccountAddress[] listAddressesWithUnspentOutputs() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new AddressesWithUnspentOutputs());

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    public OutputData[] listOutputs(Outputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public Transaction[] listPendingTransactions() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new PendingTransactions());

        Transaction[] transactions = new Transaction[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactions[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Transaction.class);

        return transactions;
    }

    public Transaction[] listTransactions() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new Transactions());

        Transaction[] transactions = new Transaction[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactions[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Transaction.class);

        return transactions;
    }

    public OutputData[] listUnspentOutputs(UnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(method);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    public TaggedDataPayload meltNativeToken(DecreaseNativeTokenSupply method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), TaggedDataPayload.class);
    }

    public String minimumRequiredStorageDeposit(MinimumRequiredStorageDeposit method) throws WalletException {
        return callAccountMethod(method).getAsJsonPrimitive().getAsString();
    }

    public MintTokenTransaction mintNativeToken(MintNativeToken method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), MintTokenTransaction.class);
    }

    public Transaction mintNfts(MintNfts method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public AccountBalance getBalance() throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(new GetBalance()), AccountBalance.class);
    }

    public Output prepareOutput(PrepareOutput method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Output.class);
    }

    public PreparedTransactionData prepareTransaction(PrepareTransaction method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), PreparedTransactionData.class);
    }

    public PreparedTransactionData prepareSendAmount(PrepareSendAmount method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), PreparedTransactionData.class);
    }

    public AccountBalance syncAccount(SyncAccount method) throws WalletException {
        AccountBalance balance = CustomGson.get().fromJson(callAccountMethod(method), AccountBalance.class);
        return balance;
    }

    public Transaction sendAmount(SendAmount method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction sendMicroTransaction(SendMicroTransaction method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction sendNativeTokens(SendNativeTokens method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction sendNft(SendNft method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public void setAlias(SetAlias method) throws WalletException {
        callAccountMethod(method);
    }

    public Transaction sendOutputs(SendOutputs method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction signTransactionEssence(SignTransactionEssence method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction submitAndStoreTransaction(SubmitAndStoreTransaction method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

    public Transaction claimOutputs(ClaimOutputs method) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(method), Transaction.class);
    }

}

class AccountHandleAdapter implements JsonSerializer<AccountHandle> {
    @Override
    public JsonElement serialize(AccountHandle src, Type typeOfSrc, JsonSerializationContext context) {
        try {
            return CustomGson.get().toJsonTree(src.getAccountCopy());
        } catch (WalletException e) {
            throw new RuntimeException(e);
        }
    }
}

