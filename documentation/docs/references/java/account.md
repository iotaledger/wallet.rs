---
description: Official IOTA Wallet Library Software Rust API reference.
image: /img/logo/wallet_light.png
keywords:
- api
- rust
- cargo
- crate
---
# Account API Reference

# Documentation

## `public int getIndex() throws WalletException`

Returns the index of the account

* **Returns:** The index of the account.

## `public int getCoinType() throws WalletException`

Returns the coin type that is configured with the account

* **Returns:** The coin type of the account.

## `public String getAlias() throws WalletException`

Returns the alias of the account

* **Returns:** The alias of the account.

## `public AccountAddress[] getPublicAddresses() throws WalletException`

Returns an array of all public addresses of the account.

* **Returns:** An array of AccountAddress objects.

## `public AccountAddress[] getInternalAddresses() throws WalletException`

Returns an array of all internal addresses of the account.

* **Returns:** An array of AccountAddress objects.

## `public Set<OutputId> getLockedOutputs() throws WalletException`

Returns all locked outputs of the account.

* **Returns:** A set of OutputIds

## `public Map<TransactionId, Map.Entry<TransactionPayload, OutputResponse[]>> getIncomingTransactions() throws WalletException`

Returns all incoming transactions for the account.

* **Returns:** All incoming transactions for the account.

## `public Account getAccountCopy() throws WalletException`

Get a snapshot of the current account state.

* **Returns:** A copy of the account.

## `public Output buildAliasOutput(BuildAliasOutput options) throws WalletException`

Builds an alias output.

* **Parameters:** `options` — The options to call.
* **Returns:** The built output.

## `public Output buildBasicOutput(BuildBasicOutput options) throws WalletException`

Builds a basic output.

* **Parameters:** `options` — The options to call.
* **Returns:** The built output.

## `public Output buildFoundryOutput(BuildFoundryOutput options) throws WalletException`

Builds a foundry output.

* **Parameters:** `options` — The options to call.
* **Returns:** The built output.

## `public Output buildNftOutput(BuildNftOutput options) throws WalletException`

Builds a NFT output.

* **Parameters:** `options` — The options to call.
* **Returns:** The built output.

## `public Transaction burnNativeToken(BurnNativeToken options) throws WalletException`

Burns native tokens for the account.

* **Parameters:** `options` — The options to be called.
* **Returns:** A transaction object.

## `public Transaction burnNft(BurnNft options) throws WalletException`

Burns a NFT.

* **Parameters:** `options` — The options.
* **Returns:** The transaction that destroyed the alias.

## `public Transaction consolidateOutputs(ConsolidateOutputs options) throws WalletException`

Destroy an alias.

* **Parameters:** `options` — The options.
* **Returns:** The transaction that destroyed the alias.

## `public Transaction destroyAlias(DestroyAlias options) throws WalletException`

Destroy an alias.

* **Parameters:** `options` — The options.
* **Returns:** The transaction that destroyed the alias.

## `public Transaction destroyFoundry(DestroyFoundry options) throws WalletException`

Destroy a foundry.

* **Parameters:** `options` — The options.
* **Returns:** The transaction that destroyed the foundry.

## `public AccountAddress[] generateAddresses(GenerateAddresses options) throws WalletException`

Generate addresses.

* **Parameters:** `options` — The options.
* **Returns:** The generated addresses.

## `public OutputData getOutput(GetOutput options) throws WalletException`

Get a specific output.

* **Parameters:** `options` — The options.
* **Returns:** The given output.

## `public Output getFoundryOutput(GetFoundryOutput options) throws WalletException`

Get a specific foundry output.

* **Parameters:** `options` — The options.
* **Returns:** The given output.

## `public Output[] getOutputsWithAdditionalUnlockConditions(GetOutputsWithAdditionalUnlockConditions options) throws WalletException`

Get all outputs with additional unlock conditions.

* **Parameters:** `options` — The options.
* **Returns:** The given transaction.

## `public Transaction getTransaction(GetTransaction options) throws WalletException`

Get a specific transaction.

* **Parameters:** `options` — The options.
* **Returns:** The given transaction.

## `public JsonElement getIncomingTransactionData(GetIncomingTransactionData options) throws WalletException`

Get the transaction with inputs of an incoming transaction stored in the account. List might not be complete, if the node pruned the data already.

* **Parameters:** `options` — The options.
* **Returns:** A JsonElement object.

## `public AccountAddress[] getAddresses() throws WalletException`

Returns all the addresses of the account.

## `public AccountAddress[] getAddressesWithUnspentOutputs() throws WalletException`

Returns all the unspent outputs of the account.

## `public OutputData[] getOutputs(Outputs options) throws WalletException`

Returns all the outputs of the account.

## `public Transaction[] getPendingTransactions() throws WalletException`

Returns all the pending transactions created by account.

## `public Transaction[] getTransactions() throws WalletException`

Returns all the transactions created by the account.

## `public OutputData[] getUnspentOutputs(UnspentOutputs options) throws WalletException`

Returns all unspent outputs.

* **Parameters:** `options` — The options.

## `public TaggedDataPayload meltNativeToken(DecreaseNativeTokenSupply options) throws WalletException`

Melts a Native Token.

* **Parameters:** `options` — The options.

## `public String minimumRequiredStorageDeposit(MinimumRequiredStorageDeposit options) throws WalletException`

Calculates the minimum required storage deposit for an output.

* **Parameters:** `options` — The options.

## `public MintTokenTransaction mintNativeToken(MintNativeToken options) throws WalletException`

Mints Native Tokens.

* **Parameters:** `options` — The options.

## `public Transaction mintNfts(MintNfts options) throws WalletException`

Mints NFTs.

* **Parameters:** `options` — The options.

## `public AccountBalance getBalance() throws WalletException`

Gets the balance of the account.

## `public Output prepareOutput(PrepareOutput options) throws WalletException`

Prepares an output.

* **Parameters:** `options` — The options.

## `public PreparedTransactionData prepareTransaction(PrepareTransaction options) throws WalletException`

Prepares a transaction.

* **Parameters:** `options` — The options.

## `public PreparedTransactionData prepareSendAmount(PrepareSendAmount options) throws WalletException`

Prepares a transaction.

* **Parameters:** `options` — The options.

## `public AccountBalance syncAccount(SyncAccount options) throws WalletException`

Sync the account by fetching new information from the nodes. Will also retry pending transactions if necessary.

* **Parameters:** `options` — The options.

## `public Transaction sendAmount(SendAmount options) throws WalletException`

Sends an amount.

* **Parameters:** `options` — The options.

## `public Transaction sendMicroTransaction(SendMicroTransaction options) throws WalletException`

Sends a micro transaction.

* **Parameters:** `options` — The options.

## `public Transaction sendNativeTokens(SendNativeTokens options) throws WalletException`

Sends Native Tokens.

* **Parameters:** `options` — The options.

## `public Transaction sendNft(SendNft options) throws WalletException`

Sends a NFT.

* **Parameters:** `options` — The options.

## `public void setAlias(SetAlias options) throws WalletException`

Set the alias of the account.

* **Parameters:** `options` — The options.

## `public Transaction sendOutputs(SendOutputs options) throws WalletException`

Send outputs in a transaction.

* **Parameters:** `options` — The options.
* **Returns:** The transaction.

## `public Transaction signTransactionEssence(SignTransactionEssence options) throws WalletException`

Signs a transaction essence.

* **Parameters:** `options` — The options.
* **Returns:** The signed transaction.

## `public Transaction submitAndStoreTransaction(SubmitAndStoreTransaction options) throws WalletException`

Submits and stores a transaction.

* **Parameters:** `options` — The options.
* **Returns:** The submitted and stored transaction.

## `public Transaction claimOutputs(ClaimOutputs options) throws WalletException`

This function claims all unclaimed outputs for the account.

* **Parameters:** `options` — The options.
* **Returns:** A transaction object.

## `public Transaction createAliasOutput(CreateAliasOutput options) throws WalletException`

Creates an alias output.

* **Parameters:** `options` — The options.
* **Returns:** A transaction object.