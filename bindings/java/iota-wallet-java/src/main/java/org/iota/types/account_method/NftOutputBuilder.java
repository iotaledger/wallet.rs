package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;
import org.iota.types.Feature;
import org.iota.types.JsonUtils;
import org.iota.types.NativeToken;
import org.iota.types.UnlockCondition;
import org.iota.types.ids.NftId;

public class NftOutputBuilder implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private NftId nftId;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public NftOutputBuilder withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public NftOutputBuilder withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public NftOutputBuilder withNftId(NftId nftId) {
        this.nftId = nftId;
        return this;
    }

    public NftOutputBuilder withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public NftOutputBuilder withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    public NftOutputBuilder withImmutableFeatures(Feature[] immutableFeatures) {
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