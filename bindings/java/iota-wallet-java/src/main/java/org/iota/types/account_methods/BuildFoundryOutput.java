package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.JsonUtils;
import org.iota.types.*;

public class BuildFoundryOutput implements AccountMethod {

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

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("amount", amount);
        o.add("nativeTokens", JsonUtils.toJson(nativeTokens));
        o.addProperty("serialNumber", serialNumber);
        o.add("tokenScheme", tokenScheme != null ? tokenScheme.toJson() : null);
        o.add("unlockConditions", JsonUtils.toJson(unlockConditions));
        o.add("features", JsonUtils.toJson(features));
        o.add("immutableFeatures", JsonUtils.toJson(immutableFeatures));

        return o;
    }
}