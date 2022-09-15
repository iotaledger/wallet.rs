package org.iota.types.account_methods;

/// Try to claim outputs.
public class TryClaimOutputs implements AccountMethod {

    private OutputsToClaim outputsToClaim;

    public enum OutputsToClaim {
        None,
        MicroTransactions,
        NativeTokens,
        Nfts,
        All,
    }

    public TryClaimOutputs withOutputsToClaim(OutputsToClaim outputsToClaim) {
        this.outputsToClaim = outputsToClaim;
        return this;
    }
}

