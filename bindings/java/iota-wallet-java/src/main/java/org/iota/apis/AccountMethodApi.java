package org.iota.apis;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import org.iota.types.*;
import org.iota.types.account_method.*;
import org.iota.types.ids.account.AccountAlias;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.ids.account.AccountIndex;

public class AccountMethodApi extends BaseApi {

    public AccountMethodApi(WalletConfig walletConfig) {
        super(walletConfig);
    }

    private JsonElement callAccountMethod(AccountIdentifier accountIdentifier, AccountMethod method) throws WalletException {
        JsonPrimitive p = null;

        if (accountIdentifier instanceof AccountIndex) {
            p = new JsonPrimitive(((AccountIndex) accountIdentifier).getAccountIndex());
        } else if (accountIdentifier instanceof AccountAlias) {
            p = new JsonPrimitive(((AccountAlias) accountIdentifier).getAccountAlias());
        } else throw new RuntimeException("unknown account identifier");

        JsonObject accountMethod = new JsonObject();
        accountMethod.addProperty("name", method.getClass().getSimpleName());
        accountMethod.add("data", method.toJson());

        JsonObject o = new JsonObject();
        o.add("accountId", p);
        o.add("method", accountMethod);

        JsonElement responsePayload = callBaseApi(new ClientCommand("CallAccountMethod", o));

        return responsePayload;
    }

    public Output buildAliasOutput(AccountIdentifier accountIdentifier, AliasOutputBuilder aliasOutputBuilder) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, aliasOutputBuilder);
        return new Output(responsePayload);
    }

    public Output buildBasicOutput(AccountIdentifier accountIdentifier, BasicOutputBuilder basicOutputBuilder) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, basicOutputBuilder);
        return new Output(responsePayload);
    }

    public Output buildFoundryOutput(AccountIdentifier accountIdentifier, FoundryOutputBuilder foundryOutputBuilder) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, foundryOutputBuilder);
        return new Output(responsePayload);
    }

    public Output buildNftOutput(AccountIdentifier accountIdentifier, NftOutputBuilder nftOutputBuilder) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, nftOutputBuilder);
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
            accountAddress[i] = new AccountAddress(responsePayload.get(i).getAsJsonObject());

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

    // TODO: dont return JSON
    public JsonObject getIncomingTransactionData(AccountIdentifier accountIdentifier, GetIncomingTransactionData getIncomingTransactionData) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, getIncomingTransactionData);
        return responsePayload;
    }

    public AccountAddress[] listAddresses(AccountIdentifier accountIdentifier, ListAddresses method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = new AccountAddress(responsePayload.get(i).getAsJsonObject());

        return addresses;
    }

    public AccountAddress[] listAddressesWithUnspentOutputs(AccountIdentifier accountIdentifier, ListAddressesWithUnspentOutputs method) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(accountIdentifier, method);

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = new AccountAddress(responsePayload.get(i).getAsJsonObject());

        return addresses;
    }

    public JsonObject listOutputs(AccountIdentifier accountIdentifier, ListOutputs method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);

        return responsePayload;
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

    public JsonObject listUnspentOutputs(AccountIdentifier accountIdentifier, ListUnspentOutputs method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);

        return responsePayload;
    }

    public TaggedDataPayload meltNativeToken(AccountIdentifier accountIdentifier, MeltNativeToken method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TaggedDataPayload(responsePayload);
    }

    public String minimumRequiredStorageDeposit(AccountIdentifier accountIdentifier, MinimumRequiredStorageDeposit method) throws WalletException {
        String responsePayload = callAccountMethod(accountIdentifier, method).getAsJsonPrimitive().getAsString();
        return responsePayload;
    }

    public TransactionPayload mintNfts(AccountIdentifier accountIdentifier, MintNfts method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

    public TransactionPayload getBalance(AccountIdentifier accountIdentifier, MintNfts method) throws WalletException {
        JsonObject responsePayload = (JsonObject) callAccountMethod(accountIdentifier, method);
        return new TransactionPayload(responsePayload);
    }

}