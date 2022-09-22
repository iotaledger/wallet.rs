package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.UnsignedByte;
import org.iota.types.features.Feature;
import org.iota.types.ids.AliasId;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build an AliasOutput.
public class BuildAliasOutput implements AccountMethod {

    // If not provided, minimum storage deposit will be used
    private String amount;
    private NativeToken[] nativeTokens;
    private AliasId aliasId;
    private Integer stateIndex;
    private UnsignedByte[] stateMetadata;
    private Integer foundryCounter;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public BuildAliasOutput withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public BuildAliasOutput withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public BuildAliasOutput withAliasId(AliasId aliasId) {
        this.aliasId = aliasId;
        return this;
    }

    public BuildAliasOutput withStateIndex(Integer stateIndex) {
        this.stateIndex = stateIndex;
        return this;
    }

    public BuildAliasOutput withStateMetadata(UnsignedByte[] stateMetadata) {
        this.stateMetadata = stateMetadata;
        return this;
    }

    public BuildAliasOutput withFoundryCounter(Integer foundryCounter) {
        this.foundryCounter = foundryCounter;
        return this;
    }

    public BuildAliasOutput withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public BuildAliasOutput withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    public BuildAliasOutput withImmutableFeatures(Feature[] immutableFeatures) {
        this.immutableFeatures = immutableFeatures;
        return this;
    }
}