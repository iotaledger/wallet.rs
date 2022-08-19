// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota;

import org.iota.apis.AccountMethodApi;
import org.iota.apis.WalletApi;
import org.iota.types.*;
import org.iota.types.account_method.*;
import org.iota.types.ids.account.AccountIdentifier;

public class Wallet {

    private WalletApi walletApi;
    private AccountMethodApi accountMethodApi;

    public Wallet(WalletConfig config) {
        walletApi = new WalletApi(config);
    }

    public void removeLatestAccount() throws WalletException {
        walletApi.removeLatestAccount();
    }

    public Account createAccount(String alias) throws WalletException {
        return walletApi.createAccount(alias);
    }

    public Account getAccount(AccountIdentifier accountIdentifier) throws WalletException {
        return walletApi.getAccount(accountIdentifier);
    }

    public Account[] getAccounts() throws WalletException {
        return walletApi.getAccounts();
    }

    // Account methods

    public Output buildAliasOutput(AccountIdentifier accountIdentifier, AliasOutputBuilder aliasOutputBuilder) throws WalletException {
        return accountMethodApi.buildAliasOutput(accountIdentifier, aliasOutputBuilder);
    }

    public Output buildBasicOutput(AccountIdentifier accountIdentifier, BasicOutputBuilder basicOutputBuilder) throws WalletException {
        return accountMethodApi.buildBasicOutput(accountIdentifier, basicOutputBuilder);
    }

    public Output buildFoundryOutput(AccountIdentifier accountIdentifier, FoundryOutputBuilder foundryOutputBuilder) throws WalletException {
        return accountMethodApi.buildFoundryOutput(accountIdentifier, foundryOutputBuilder);
    }

    public Output buildNftOutput(AccountIdentifier accountIdentifier, NftOutputBuilder nftOutputBuilder) throws WalletException {
        return accountMethodApi.buildNftOutput(accountIdentifier, nftOutputBuilder);
    }

    public TaggedDataPayload burnNativeToken(AccountIdentifier accountIdentifier, BurnNativeToken burnNativeToken) throws WalletException {
        return accountMethodApi.burnNativeToken(accountIdentifier, burnNativeToken);
    }

    public TaggedDataPayload burnNativeToken(AccountIdentifier accountIdentifier, BurnNft burnNft) throws WalletException {
        return accountMethodApi.burnNft(accountIdentifier, burnNft);
    }

}