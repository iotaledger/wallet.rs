package org.iota.types.outputs;

import org.iota.types.NativeToken;
import org.iota.types.account_methods.AccountMethod;
import org.iota.types.features.Feature;
import org.iota.types.token_scheme.TokenScheme;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build a FoundryOutput.
public class FoundryOutput extends Output {

    private int type = 5;
    // Amount of IOTA tokens held by the output.
    private String amount;
    // Native tokens held by the output.
    private NativeToken[] nativeTokens;
    // The serial number of the foundry with respect to the controlling alias.
    private int serialNumber;
    private TokenScheme tokenScheme;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public FoundryOutput(String amount, NativeToken[] nativeTokens, int serialNumber, TokenScheme tokenScheme, UnlockCondition[] unlockConditions, Feature[] features, Feature[] immutableFeatures) {
        this.amount = amount;
        this.nativeTokens = nativeTokens;
        this.serialNumber = serialNumber;
        this.tokenScheme = tokenScheme;
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

    public int getSerialNumber() {
        return serialNumber;
    }

    public TokenScheme getTokenScheme() {
        return tokenScheme;
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