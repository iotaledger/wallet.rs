package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.ids.NftId;
import org.iota.types.unlock_conditions.UnlockCondition;

public class BuildNftOutput implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private NftId nftId;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

}