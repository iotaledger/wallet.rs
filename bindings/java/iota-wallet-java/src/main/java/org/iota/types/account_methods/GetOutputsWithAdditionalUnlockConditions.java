package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

public class GetOutputsWithAdditionalUnlockConditions implements AccountMethod {

    private OutputsToClaim outputsToClaim;

    public enum OutputsToClaim {
        None,
        MicroTransactions,
        NativeTokens,
        Nfts,
        All,
    }

}