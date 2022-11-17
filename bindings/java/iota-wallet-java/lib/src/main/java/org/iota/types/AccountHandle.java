// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.Wallet;
import org.iota.api.WalletCommand;
import org.iota.api.CustomGson;
import org.iota.types.account_methods.*;
import org.iota.types.exceptions.WalletException;
import org.iota.types.ids.OutputId;
import org.iota.types.ids.TransactionId;
import org.iota.types.ids.account.AccountIdentifier;
import org.iota.types.outputs.Output;
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

    /**
     * Returns the index of the account
     *
     * @return The index of the account.
     */
    public int getIndex() throws WalletException {
        return getAccountCopy().getIndex();
    }

    /**
     * Returns the coin type that is configured with the account
     *
     * @return The coin type of the account.
     */
    public int getCoinType() throws WalletException {
        return getAccountCopy().getCoinType();
    }

    /**
     * Returns the alias of the account
     *
     * @return The alias of the account.
     */
    public String getAlias() throws WalletException {
        return getAccountCopy().getAlias();
    }

    /**
     * Returns an array of all public addresses of the account.
     *
     * @return An array of AccountAddress objects.
     */
    public AccountAddress[] getPublicAddresses() throws WalletException {
        return getAccountCopy().getPublicAddresses();
    }

    /**
     * Returns an array of all internal addresses of the account.
     *
     * @return An array of AccountAddress objects.
     */
    public AccountAddress[] getInternalAddresses() throws WalletException {
        return getAccountCopy().getInternalAddresses();
    }

    /**
     * Returns all locked outputs of the account.
     *
     * @return A set of OutputIds
     */
    public Set<OutputId> getLockedOutputs() throws WalletException {
        return getAccountCopy().getLockedOutputs();
    }

    /**
     * Returns all incoming transactions for the account.
     *
     * @return All incoming transactions for the account.
     */
    public Map<TransactionId, Map.Entry<TransactionPayload, OutputResponse[]>> getIncomingTransactions() throws WalletException {
        return getAccountCopy().getIncomingTransactions();
    }

    /**
     * Get a snapshot of the current account state.
     *
     * @return A copy of the account.
     */
    public Account getAccountCopy() throws WalletException {
        return CustomGson.get().fromJson(callBaseApi(new WalletCommand("getAccount", CustomGson.get().toJsonTree(accountIdentifier))), Account.class);
    }

    // Account Method APIs

    private JsonElement callAccountMethod(AccountMethod accountMethod) throws WalletException {
        JsonObject method = new JsonObject();

        String methodName = accountMethod.getClass().getSimpleName();
        method.addProperty("name", methodName.substring(0, 1).toLowerCase() + methodName.substring(1));

        JsonElement data = CustomGson.get().toJsonTree(accountMethod);
        if(data.toString().equals("{}"))
            method.add("data", null);
        else
            method.add("data", data);

        JsonObject o = new JsonObject();
        o.add("accountId", CustomGson.get().toJsonTree(accountIdentifier));
        o.add("method", method);

        return callBaseApi(new WalletCommand("callAccountMethod", o));
    }

    /**
     * Builds an alias output.
     *
     * @param options The options to call.
     * @return The built output.
     */
    public Output buildAliasOutput(BuildAliasOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Builds a basic output.
     *
     * @param options The options to call.
     * @return The built output.
     */
    public Output buildBasicOutput(BuildBasicOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Builds a foundry output.
     *
     * @param options The options to call.
     * @return The built output.
     */
    public Output buildFoundryOutput(BuildFoundryOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Builds a NFT output.
     *
     * @param options The options to call.
     * @return The built output.
     */
    public Output buildNftOutput(BuildNftOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Burns native tokens for the account.
     *
     * @param options The options to be called.
     * @return A transaction object.
     */
    public Transaction burnNativeToken(BurnNativeToken options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Burns a NFT.
     *
     * @param options The options.
     * @return The transaction that destroyed the alias.
     */
    public Transaction burnNft(BurnNft options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Destroy an alias.
     *
     * @param options The options.
     * @return The transaction that destroyed the alias.
     */
    public Transaction consolidateOutputs(ConsolidateOutputs options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Destroy an alias.
     *
     * @param options The options.
     * @return The transaction that destroyed the alias.
     */
    public Transaction destroyAlias(DestroyAlias options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Destroy a foundry.
     *
     * @param options The options.
     * @return The transaction that destroyed the foundry.
     */
    public Transaction destroyFoundry(DestroyFoundry options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Generate addresses.
     *
     * @param options The options.
     * @return The generated addresses.
     */
    public AccountAddress[] generateAddresses(GenerateAddresses options) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(options);

        AccountAddress[] accountAddress = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            accountAddress[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return accountAddress;
    }

    /**
     * Get a specific output.
     *
     * @param options The options.
     * @return The given output.
     */
    public OutputData getOutput(GetOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), OutputData.class);
    }

    /**
     * Get a specific foundry output.
     *
     * @param options The options.
     * @return The given output.
     */
    public Output getFoundryOutput(GetFoundryOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Get all outputs with additional unlock conditions.
     *
     * @param options The options.
     * @return The given transaction.
     */
    public Output[] getOutputsWithAdditionalUnlockConditions(GetOutputsWithAdditionalUnlockConditions options) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(options);

        Output[] outputs = new Output[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputs[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Output.class);

        return outputs;
    }

    /**
     * Get a specific transaction.
     *
     * @param options The options.
     * @return The given transaction.
     */
    public Transaction getTransaction(GetTransaction options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }


    /**
     * Get the transaction with inputs of an incoming transaction stored in the account.
     * List might not be complete, if the node pruned the data already.
     *
     * @param options The options.
     * @return A JsonElement object.
     */
    public JsonElement getIncomingTransactionData(GetIncomingTransactionData options) throws WalletException {
        return callAccountMethod(options);
    }

    /**
     * Returns all the addresses of the account.
     */
    public AccountAddress[] getAddresses() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new Addresses());

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    /**
     * Returns all the unspent outputs of the account.
     */
    public AccountAddress[] getAddressesWithUnspentOutputs() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new AddressesWithUnspentOutputs());

        AccountAddress[] addresses = new AccountAddress[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            addresses[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), AccountAddress.class);

        return addresses;
    }

    /**
     * Returns all the outputs of the account.
     */
    public OutputData[] getOutputs(Outputs options) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(options);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    /**
     * Returns all the pending transactions created by account.
     */
    public Transaction[] getPendingTransactions() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new PendingTransactions());

        Transaction[] transactions = new Transaction[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactions[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Transaction.class);

        return transactions;
    }

    /**
     * Returns all the transactions created by the account.
     */
    public Transaction[] getTransactions() throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(new Transactions());

        Transaction[] transactions = new Transaction[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            transactions[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), Transaction.class);

        return transactions;
    }

    /**
     * Returns all unspent outputs.
     *
     * @param options The options.
     */
    public OutputData[] getUnspentOutputs(UnspentOutputs options) throws WalletException {
        JsonArray responsePayload = (JsonArray) callAccountMethod(options);

        OutputData[] outputsData = new OutputData[responsePayload.size()];
        for (int i = 0; i < responsePayload.size(); i++)
            outputsData[i] = CustomGson.get().fromJson(responsePayload.get(i).getAsJsonObject(), OutputData.class);

        return outputsData;
    }

    /**
     * Melts a Native Token.
     *
     * @param options The options.
     */
    public Transaction meltNativeToken(DecreaseNativeTokenSupply options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Calculates the minimum required storage deposit for an output.
     *
     * @param options The options.
     */
    public String minimumRequiredStorageDeposit(MinimumRequiredStorageDeposit options) throws WalletException {
        return callAccountMethod(options).getAsJsonPrimitive().getAsString();
    }

    /**
     * Mints Native Tokens.
     *
     * @param options The options.
     */
    public MintTokenTransaction mintNativeToken(MintNativeToken options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), MintTokenTransaction.class);
    }

    /**
     * Mints NFTs.
     *
     * @param options The options.
     */
    public Transaction mintNfts(MintNfts options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Gets the balance of the account.
     */
    public AccountBalance getBalance() throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(new GetBalance()), AccountBalance.class);
    }

    /**
     * Prepares an output.
     *
     * @param options The options.
     */
    public Output prepareOutput(PrepareOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Output.class);
    }

    /**
     * Prepares a transaction.
     *
     * @param options The options.
     */
    public PreparedTransactionData prepareTransaction(PrepareTransaction options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), PreparedTransactionData.class);
    }

    /**
     * Prepares a transaction.
     *
     * @param options The options.
     */
    public PreparedTransactionData prepareSendAmount(PrepareSendAmount options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), PreparedTransactionData.class);
    }

    /**
     * Sync the account by fetching new information from the nodes. Will also retry pending transactions if necessary.
     *
     * @param options The options.
     */
    public AccountBalance syncAccount(SyncAccount options) throws WalletException {
        AccountBalance balance = CustomGson.get().fromJson(callAccountMethod(options), AccountBalance.class);
        return balance;
    }

    /**
     * Sends an amount.
     *
     * @param options The options.
     */
    public Transaction sendAmount(SendAmount options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Sends a micro transaction.
     *
     * @param options The options.
     */
    public Transaction sendMicroTransaction(SendMicroTransaction options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Sends Native Tokens.
     *
     * @param options The options.
     */
    public Transaction sendNativeTokens(SendNativeTokens options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Sends a NFT.
     *
     * @param options The options.
     */
    public Transaction sendNft(SendNft options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Set the alias of the account.
     *
     * @param options The options.
     */
    public void setAlias(SetAlias options) throws WalletException {
        callAccountMethod(options);
    }

    /**
     * Send outputs in a transaction.
     *
     * @param options The options.
     * @return The transaction.
     */
    public Transaction sendOutputs(SendOutputs options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Signs a transaction essence.
     *
     * @param options The options.
     * @return The signed transaction.
     */
    public Transaction signTransactionEssence(SignTransactionEssence options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Submits and stores a transaction.
     *
     * @param options The options.
     * @return The submitted and stored transaction.
     */
    public Transaction submitAndStoreTransaction(SubmitAndStoreTransaction options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }


    /**
     * This function claims all unclaimed outputs for the account.
     *
     * @param options The options.
     * @return A transaction object.
     */
    public Transaction claimOutputs(ClaimOutputs options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
    }

    /**
     * Creates an alias output.
     *
     * @param options The options.
     * @return A transaction object.
     */
    public Transaction createAliasOutput(CreateAliasOutput options) throws WalletException {
        return CustomGson.get().fromJson(callAccountMethod(options), Transaction.class);
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

