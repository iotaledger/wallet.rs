package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.ids.AliasId;
import org.iota.types.unlock_conditions.UnlockCondition;

public class BuildAliasOutput implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private AliasId aliasId;
    private Integer stateIndex;
    private byte[] stateMetadata;
    private Integer foundryCounter;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

}