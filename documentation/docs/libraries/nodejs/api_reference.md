## Classes

<dl>
<dt><a href="#Account">Account</a></dt>
<dd><p>The Account class.</p></dd>
<dt><a href="#AccountManager">AccountManager</a></dt>
<dd><p>The AccountManager class.</p></dd>
</dl>

## Functions

<dl>
<dt><a href="#initLogger">initLogger()</a></dt>
<dd><p>Function to create wallet logs</p></dd>
</dl>

<a name="Account"></a>

## Account
<p>The Account class.</p>

**Kind**: global class  

* [Account](#Account)
    * [.buildAliasOutput(data)](#Account+buildAliasOutput) ⇒
    * [.buildBasicOutput(data)](#Account+buildBasicOutput) ⇒
    * [.buildFoundryOutput(data)](#Account+buildFoundryOutput) ⇒
    * [.buildNftOutput(data)](#Account+buildNftOutput) ⇒
    * [.claimOutputs(outputIds)](#Account+claimOutputs) ⇒
    * [.consolidateOutputs(force, outputConsolidationThreshold)](#Account+consolidateOutputs) ⇒
    * [.generateAddress(options)](#Account+generateAddress) ⇒
    * [.generateAddresses(amount, options)](#Account+generateAddresses) ⇒
    * [.getAlias()](#Account+getAlias) ⇒
    * [.getBalance()](#Account+getBalance) ⇒
    * [.getOutput(outputId)](#Account+getOutput) ⇒
    * [.getFoundryOutput(tokenId)](#Account+getFoundryOutput) ⇒
    * [.getOutputsWithAdditionalUnlockConditions(outputs)](#Account+getOutputsWithAdditionalUnlockConditions) ⇒
    * [.getTransaction(transactionId)](#Account+getTransaction) ⇒
    * [.listAddresses()](#Account+listAddresses) ⇒
    * [.listAddressesWithUnspentOutputs()](#Account+listAddressesWithUnspentOutputs) ⇒
    * [.listOutputs()](#Account+listOutputs) ⇒
    * [.listPendingTransactions()](#Account+listPendingTransactions) ⇒
    * [.listTransactions()](#Account+listTransactions) ⇒
    * [.listUnspentOutputs()](#Account+listUnspentOutputs) ⇒
    * [.minimumRequiredStorageDeposit(output)](#Account+minimumRequiredStorageDeposit) ⇒
    * [.mintNativeToken(nativeTokenOptions, transactionOptions)](#Account+mintNativeToken) ⇒
    * [.mintNfts(nftsOptions, transactionOptions)](#Account+mintNfts) ⇒
    * [.prepareOutput(options, transactionOptions)](#Account+prepareOutput) ⇒
    * [.prepareSendAmount(addressesWithAmount, options)](#Account+prepareSendAmount) ⇒
    * [.prepareTransaction(outputs, options)](#Account+prepareTransaction) ⇒
    * [.sendAmount(addressesWithAmount, transactionOptions)](#Account+sendAmount) ⇒
    * [.sendMicroTransaction(addressesWithMicroAmount, transactionOptions)](#Account+sendMicroTransaction) ⇒
    * [.sendNativeTokens(addressesNativeTokens, transactionOptions)](#Account+sendNativeTokens) ⇒
    * [.sendNft(addressesAndNftIds, transactionOptions)](#Account+sendNft) ⇒
    * [.sendOutputs(outputs, transactionOptions)](#Account+sendOutputs) ⇒
    * [.signTransactionEssence(preparedTransactionData)](#Account+signTransactionEssence) ⇒
    * [.submitAndStoreTransaction(signedTransactionData)](#Account+submitAndStoreTransaction) ⇒
    * [.sync(options)](#Account+sync) ⇒
    * [.tryClaimOutputs(outputsToClaim)](#Account+tryClaimOutputs) ⇒

<a name="Account+buildAliasOutput"></a>

### account.buildAliasOutput(data) ⇒
<p>Build an <code>AliasOutput</code>.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The built <code>AliasOutput</code>.</p>  

| Param | Description |
| --- | --- |
| data | <p>Options for building an <code>AliasOutput</code>.</p> |

<a name="Account+buildBasicOutput"></a>

### account.buildBasicOutput(data) ⇒
<p>Build a <code>BasicOutput</code>.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The built <code>BasicOutput</code>.</p>  

| Param | Description |
| --- | --- |
| data | <p>Options for building a <code>BasicOutput</code>.</p> |

<a name="Account+buildFoundryOutput"></a>

### account.buildFoundryOutput(data) ⇒
<p>Build a <code>FoundryOutput</code>.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The built <code>FoundryOutput</code>.</p>  

| Param | Description |
| --- | --- |
| data | <p>Options for building a <code>FoundryOutput</code>.</p> |

<a name="Account+buildNftOutput"></a>

### account.buildNftOutput(data) ⇒
<p>Build an <code>NftOutput</code>.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The built <code>NftOutput</code>.</p>  

| Param | Description |
| --- | --- |
| data | <p>Options for building an <code>NftOutput</code>.</p> |

<a name="Account+claimOutputs"></a>

### account.claimOutputs(outputIds) ⇒
<p>Claim basic or nft outputs that have additional unlock conditions
to their <code>AddressUnlockCondition</code> from the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The resulting transactions.</p>  

| Param | Description |
| --- | --- |
| outputIds | <p>The outputs to claim.</p> |

<a name="Account+consolidateOutputs"></a>

### account.consolidateOutputs(force, outputConsolidationThreshold) ⇒
<p>Consolidate basic outputs with only an <code>AddressUnlockCondition</code> from an account
by sending them to the same address again if the output amount is greater or
equal to the output consolidation threshold.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The consolidation transactions.</p>  

| Param | Description |
| --- | --- |
| force | <p>Force consolidation on addresses where the threshold isn't met.</p> |
| outputConsolidationThreshold | <p>A default threshold is used if this is omitted.</p> |

<a name="Account+generateAddress"></a>

### account.generateAddress(options) ⇒
<p>Generate a new unused address.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The address.</p>  

| Param | Description |
| --- | --- |
| options | <p>Options for address generation.</p> |

<a name="Account+generateAddresses"></a>

### account.generateAddresses(amount, options) ⇒
<p>Generate new unused addresses.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The addresses.</p>  

| Param | Description |
| --- | --- |
| amount | <p>The amount of addresses to generate.</p> |
| options | <p>Options for address generation.</p> |

<a name="Account+getAlias"></a>

### account.getAlias() ⇒
<p>Get this accounts alias.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The accounts alias.</p>  
<a name="Account+getBalance"></a>

### account.getBalance() ⇒
<p>Get the account balance.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The account balance.</p>  
<a name="Account+getOutput"></a>

### account.getOutput(outputId) ⇒
<p>Get the data for an output.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The <code>OutputData</code>.</p>  

| Param | Description |
| --- | --- |
| outputId | <p>The output to get.</p> |

<a name="Account+getFoundryOutput"></a>

### account.getFoundryOutput(tokenId) ⇒
<p>Get a <code>FoundryOutput</code> by native token ID. It will try to get the foundry from
the account, if it isn't in the account it will try to get it from the node.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The <code>FoundryOutput</code> that minted the token.</p>  

| Param | Description |
| --- | --- |
| tokenId | <p>The native token ID to get the foundry for.</p> |

<a name="Account+getOutputsWithAdditionalUnlockConditions"></a>

### account.getOutputsWithAdditionalUnlockConditions(outputs) ⇒
<p>Get outputs with additional unlock conditions.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The output IDs of the unlockable outputs.</p>  

| Param | Description |
| --- | --- |
| outputs | <p>The type of outputs to claim.</p> |

<a name="Account+getTransaction"></a>

### account.getTransaction(transactionId) ⇒
<p>Get a transaction stored in the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The transaction.</p>  

| Param | Description |
| --- | --- |
| transactionId | <p>The ID of the transaction to get.</p> |

<a name="Account+listAddresses"></a>

### account.listAddresses() ⇒
<p>List all the addresses of the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The addresses.</p>  
<a name="Account+listAddressesWithUnspentOutputs"></a>

### account.listAddressesWithUnspentOutputs() ⇒
<p>List the addresses of the account with unspent outputs.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The addresses.</p>  
<a name="Account+listOutputs"></a>

### account.listOutputs() ⇒
<p>List all outputs of the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The outputs with metadata.</p>  
<a name="Account+listPendingTransactions"></a>

### account.listPendingTransactions() ⇒
<p>List all the pending transactions of the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The transactions.</p>  
<a name="Account+listTransactions"></a>

### account.listTransactions() ⇒
<p>List all the transactions of the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The transactions.</p>  
<a name="Account+listUnspentOutputs"></a>

### account.listUnspentOutputs() ⇒
<p>List all the unspent outputs of the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The outputs with metadata.</p>  
<a name="Account+minimumRequiredStorageDeposit"></a>

### account.minimumRequiredStorageDeposit(output) ⇒
<p>Calculate the minimum required storage deposit for an output.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The amount.</p>  

| Param | Description |
| --- | --- |
| output | <p>output to calculate the deposit amount for.</p> |

<a name="Account+mintNativeToken"></a>

### account.mintNativeToken(nativeTokenOptions, transactionOptions) ⇒
<p>Mint native tokens.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The minting transaction and the token ID.</p>  

| Param | Description |
| --- | --- |
| nativeTokenOptions | <p>The options for minting tokens.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+mintNfts"></a>

### account.mintNfts(nftsOptions, transactionOptions) ⇒
<p>Mint nfts.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The minting transaction.</p>  

| Param | Description |
| --- | --- |
| nftsOptions | <p>The options for minting nfts.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+prepareOutput"></a>

### account.prepareOutput(options, transactionOptions) ⇒
<p>Prepare an output for sending, useful for offline signing.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The prepared output.</p>  

| Param | Description |
| --- | --- |
| options | <p>The options for preparing an output. If the amount is below the minimum required storage deposit, by default the remaining amount will automatically be added with a <code>StorageDepositReturn</code> <code>UnlockCondition</code>, when setting the <code>ReturnStrategy</code> to <code>gift</code>, the full minimum required storage deposit will be sent  to the recipient. When the assets contain an nft id, the data from the exisiting <code>NftOutput</code> will be used, just with the address unlock conditions replaced.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+prepareSendAmount"></a>

### account.prepareSendAmount(addressesWithAmount, options) ⇒
<p>Prepare a send amount transaction, useful for offline signing.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The prepared transaction data.</p>  

| Param | Description |
| --- | --- |
| addressesWithAmount | <p>Address with amounts to send.</p> |
| options | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+prepareTransaction"></a>

### account.prepareTransaction(outputs, options) ⇒
<p>Prepare a transaction, useful for offline signing.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The prepared transaction data.</p>  

| Param | Description |
| --- | --- |
| outputs | <p>Outputs to use in the transaction.</p> |
| options | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+sendAmount"></a>

### account.sendAmount(addressesWithAmount, transactionOptions) ⇒
<p>Send a transaction with amounts from input addresses.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| addressesWithAmount | <p>Addresses with amounts.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+sendMicroTransaction"></a>

### account.sendMicroTransaction(addressesWithMicroAmount, transactionOptions) ⇒
<p>Send a micro transaction with amount below minimum storage deposit.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| addressesWithMicroAmount | <p>Addresses with micro amounts.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+sendNativeTokens"></a>

### account.sendNativeTokens(addressesNativeTokens, transactionOptions) ⇒
<p>Send native tokens.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| addressesNativeTokens | <p>Addresses amounts and native tokens.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+sendNft"></a>

### account.sendNft(addressesAndNftIds, transactionOptions) ⇒
<p>Send nft.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| addressesAndNftIds | <p>Addresses and nft ids.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+sendOutputs"></a>

### account.sendOutputs(outputs, transactionOptions) ⇒
<p>Send outputs in a transaction.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| outputs | <p>The outputs to send.</p> |
| transactionOptions | <p>The options to define a <code>RemainderValueStrategy</code> or custom inputs.</p> |

<a name="Account+signTransactionEssence"></a>

### account.signTransactionEssence(preparedTransactionData) ⇒
<p>Sign a prepared transaction, useful for offline signing.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The signed transaction essence.</p>  

| Param | Description |
| --- | --- |
| preparedTransactionData | <p>The prepared transaction data to sign.</p> |

<a name="Account+submitAndStoreTransaction"></a>

### account.submitAndStoreTransaction(signedTransactionData) ⇒
<p>Validate the transaction, submit it to a node and store it in the account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The sent transaction.</p>  

| Param | Description |
| --- | --- |
| signedTransactionData | <p>A signed transaction to submit and store.</p> |

<a name="Account+sync"></a>

### account.sync(options) ⇒
<p>Sync the account by fetching new information from the nodes.
Will also retry pending transactions if necessary.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The account balance.</p>  

| Param | Description |
| --- | --- |
| options | <p>Optional synchronization options.</p> |

<a name="Account+tryClaimOutputs"></a>

### account.tryClaimOutputs(outputsToClaim) ⇒
<p>Try to claim basic outputs that have additional unlock conditions to
their <code>AddressUnlockCondition</code> and send them to the first address of the
account.</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
**Returns**: <p>The resulting transactions.</p>  

| Param | Description |
| --- | --- |
| outputsToClaim | <p>Outputs to try to claim.</p> |

<a name="AccountManager"></a>

## AccountManager
<p>The AccountManager class.</p>

**Kind**: global class  

* [AccountManager](#AccountManager)
    * [.bech32ToHex()](#AccountManager+bech32ToHex)
    * [.hexToBech32()](#AccountManager+hexToBech32)

<a name="AccountManager+bech32ToHex"></a>

### accountManager.bech32ToHex()
<p>Transform a bech32 encoded address to a hex encoded address</p>

**Kind**: instance method of [<code>AccountManager</code>](#AccountManager)  
<a name="AccountManager+hexToBech32"></a>

### accountManager.hexToBech32()
<p>Transform hex encoded address to bech32 encoded address. If no bech32Hrp
is provided, the AccountManager will attempt to retrieve it from the
NodeInfo. If this does not succeed, it will default to the Shimmer testnet bech32Hrp.</p>

**Kind**: instance method of [<code>AccountManager</code>](#AccountManager)  
<a name="initLogger"></a>

## initLogger()
<p>Function to create wallet logs</p>

**Kind**: global function  
