package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.token_scheme.TokenScheme;
import org.iota.types.unlock_conditions.UnlockCondition;

public class BuildFoundryOutput implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private int serialNumber;
    private TokenScheme tokenScheme;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

}