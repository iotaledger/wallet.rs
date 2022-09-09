package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.Feature;
import org.iota.JsonUtils;
import org.iota.types.NativeToken;
import org.iota.types.UnlockCondition;
import org.iota.types.ids.NftId;

public class BuildNftOutput implements AccountMethod {

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

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("amount", amount);
        o.add("nativeTokens", JsonUtils.toJson(nativeTokens));
        o.addProperty("nftId", nftId != null ? nftId.toString() : null);
        o.add("unlockConditions", JsonUtils.toJson(unlockConditions));
        o.add("features", JsonUtils.toJson(features));
        o.add("immutableFeatures", JsonUtils.toJson(immutableFeatures));

        return o;
    }

}