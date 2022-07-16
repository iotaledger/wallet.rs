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
<a name="Account+getFoundryOutput"></a>

### account.getFoundryOutput()
<p>Get a foundry output by native token ID. It will try to get the foundry from
the account, if it isn't in the account it will try to get it from the node</p>

**Kind**: instance method of [<code>Account</code>](#Account)  
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
