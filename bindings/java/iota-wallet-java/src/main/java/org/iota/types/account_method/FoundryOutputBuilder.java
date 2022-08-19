package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;
import org.iota.types.*;

public class FoundryOutputBuilder implements AccountMethod {

    private String amount;
    private NativeToken[] nativeTokens;
    private int serialNumber;
    private TokenScheme tokenScheme;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public FoundryOutputBuilder withAmount(String amount) {
        this.amount = amount;
        return this;
    }

    public FoundryOutputBuilder withNativeTokens(NativeToken[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public FoundryOutputBuilder withSerialNumber(int serialNumber) {
        this.serialNumber = serialNumber;
        return this;
    }

    public FoundryOutputBuilder withTokenScheme(TokenScheme tokenScheme) {
        this.tokenScheme = tokenScheme;
        return this;
    }

    public FoundryOutputBuilder withUnlockConditions(UnlockCondition[] unlockConditions) {
        this.unlockConditions = unlockConditions;
        return this;
    }

    public FoundryOutputBuilder withFeatures(Feature[] features) {
        this.features = features;
        return this;
    }

    public FoundryOutputBuilder withImmutableFeatures(Feature[] immutableFeatures) {
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