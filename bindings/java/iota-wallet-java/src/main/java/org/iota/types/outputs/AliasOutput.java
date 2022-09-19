package org.iota.types.outputs;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.ids.AliasId;
import org.iota.types.unlock_conditions.UnlockCondition;

public class AliasOutput extends Output {

    private int type = 4;
    // Amount of IOTA tokens held by the output.
    private String amount;
    // Native tokens held by the output.
    private NativeToken[] nativeTokens;
    // Unique identifier of the alias.
    private AliasId aliasId;
    // A counter that must increase by 1 every time the alias is state transitioned.
    private int stateIndex;
    // Metadata that can only be changed by the state controller.
    private String stateMetadata;
    // A counter that denotes the number of foundries created by this alias account.
    private int foundriesCounter;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public AliasOutput(String amount, NativeToken[] nativeTokens, AliasId aliasId, int stateIndex, String stateMetadata, int foundriesCounter, UnlockCondition[] unlockConditions, Feature[] features, Feature[] immutableFeatures) {
        this.amount = amount;
        this.nativeTokens = nativeTokens;
        this.aliasId = aliasId;
        this.stateIndex = stateIndex;
        this.stateMetadata = stateMetadata;
        this.foundriesCounter = foundriesCounter;
        this.unlockConditions = unlockConditions;
        this.features = features;
        this.immutableFeatures = immutableFeatures;
    }

    public int getType() {
        return type;
    }

    public String getAmount() {
        return amount;
    }

    public NativeToken[] getNativeTokens() {
        return nativeTokens;
    }

    public AliasId getAliasId() {
        return aliasId;
    }

    public int getStateIndex() {
        return stateIndex;
    }

    public String getStateMetadata() {
        return stateMetadata;
    }

    public int getFoundriesCounter() {
        return foundriesCounter;
    }

    public UnlockCondition[] getUnlockConditions() {
        return unlockConditions;
    }

    public Feature[] getFeatures() {
        return features;
    }

    public Feature[] getImmutableFeatures() {
        return immutableFeatures;
    }
}
