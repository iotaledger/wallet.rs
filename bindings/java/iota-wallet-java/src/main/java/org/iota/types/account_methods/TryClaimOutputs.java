package org.iota.types.account_methods;

public class TryClaimOutputs implements AccountMethod {

    private OutputsToClaim outputsToClaim;

    public enum OutputsToClaim {
        None,
        MicroTransactions,
        NativeTokens,
        Nfts,
        All,
    }

}

