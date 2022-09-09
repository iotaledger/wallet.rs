package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.JsonUtils;
import org.iota.types.*;

public class BuildBasicOutput implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;

    public BuildBasicOutput withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public BuildBasicOutput withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public BuildBasicOutput withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public BuildBasicOutput withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("amount", amount);
        o.add("nativeTokens", JsonUtils.toJson(nativeTokens));
        o.add("unlockConditions", JsonUtils.toJson(unlockConditions));
        o.add("features", JsonUtils.toJson(features));

        return o;
    }

}