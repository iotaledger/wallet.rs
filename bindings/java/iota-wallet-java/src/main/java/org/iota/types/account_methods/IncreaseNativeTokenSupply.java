package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.TokenId;

/// Mint more native token.
public class IncreaseNativeTokenSupply implements AccountMethod {

    private TokenId tokenId;
    private String mintAmount;
    private IncreaseNativeTokenSupplyOptions increaseNativeTokenSupplyOptions;
    private TransactionOptions transactionOptions;

    public static class IncreaseNativeTokenSupplyOptions {}

}