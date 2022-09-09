package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.Feature;
import org.iota.JsonUtils;
import org.iota.types.NativeToken;
import org.iota.types.UnlockCondition;
import org.iota.types.ids.AliasId;

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

    public BuildAliasOutput withStateMetadata(byte[] stateMetadata) {
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

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("amount", amount);
        o.add("nativeTokens", JsonUtils.toJson(nativeTokens));
        o.addProperty("aliasId", aliasId != null ? aliasId.toString() : null);
        o.addProperty("stateIndex", stateIndex);
        o.add("stateMetadata", JsonUtils.toJson(stateMetadata));
        o.addProperty("foundryCounter", foundryCounter);
        o.add("unlockConditions", JsonUtils.toJson(unlockConditions));
        o.add("features", JsonUtils.toJson(features));
        o.add("immutableFeatures", JsonUtils.toJson(immutableFeatures));

        return o;
    }
}