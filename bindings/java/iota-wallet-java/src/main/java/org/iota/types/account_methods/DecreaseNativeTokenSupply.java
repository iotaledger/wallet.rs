package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.TokenId;

/// Melt native tokens. This happens with the foundry output which minted them, by increasing it's
/// `melted_tokens` field.
public class DecreaseNativeTokenSupply implements AccountMethod {

    private TokenId tokenId;
    private String meltAmount;
    private TransactionOptions transactionOptions;

    public DecreaseNativeTokenSupply withTokenId(TokenId tokenId) {
        this.tokenId = tokenId;
        return this;
    }

    public DecreaseNativeTokenSupply withMeltAmount(String meltAmount) {
        this.meltAmount = meltAmount;
        return this;
    }

    public DecreaseNativeTokenSupply withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}