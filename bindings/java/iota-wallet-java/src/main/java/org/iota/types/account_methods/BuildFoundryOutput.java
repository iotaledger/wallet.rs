package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.token_scheme.TokenScheme;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build a FoundryOutput.
public class BuildFoundryOutput implements AccountMethod {

    // If not provided, minimum storage deposit will be used
    private String amount;
    private NativeToken[] nativeTokens;
    private int serialNumber;
    private TokenScheme tokenScheme;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public BuildFoundryOutput withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public BuildFoundryOutput withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public BuildFoundryOutput withSerialNumber(int serialNumber) {
        this.serialNumber = serialNumber;
        return this;
    }

    public BuildFoundryOutput withTokenScheme(TokenScheme tokenScheme) {
        this.tokenScheme = tokenScheme;
        return this;
    }

    public BuildFoundryOutput withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public BuildFoundryOutput withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    public BuildFoundryOutput withImmutableFeatures(Feature[] immutableFeatures) {
        this.immutableFeatures = immutableFeatures;
        return this;
    }
}