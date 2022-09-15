package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.ids.NftId;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build an NftOutput.
public class BuildNftOutput implements AccountMethod {

    // If not provided, minimum storage deposit will be used
    private String amount;
    private NativeToken[] nativeTokens;
    private NftId nftId;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public BuildNftOutput withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public BuildNftOutput withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public BuildNftOutput withNftId(NftId nftId) {
        this.nftId = nftId;
        return this;
    }

    public BuildNftOutput withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public BuildNftOutput withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    public BuildNftOutput withImmutableFeatures(Feature[] immutableFeatures) {
        this.immutableFeatures = immutableFeatures;
        return this;
    }
}