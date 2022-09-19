package org.iota.types.outputs;

import org.iota.types.NativeToken;
import org.iota.types.account_methods.AccountMethod;
import org.iota.types.features.Feature;
import org.iota.types.ids.NftId;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build an NftOutput.
public class NftOutput extends Output {

    private int type = 6;
    // Amount of IOTA tokens held by the output.
    private String amount;
    // Native tokens held by the output.
    private NativeToken[] nativeTokens;
    // Unique identifier of the NFT.
    private NftId nftId;
    private UnlockCondition[] unlockConditions;
    private Feature[] features;
    private Feature[] immutableFeatures;

    public NftOutput(String amount, NativeToken[] nativeTokens, NftId nftId, UnlockCondition[] unlockConditions, Feature[] features, Feature[] immutableFeatures) {
        this.amount = amount;
        this.nativeTokens = nativeTokens;
        this.nftId = nftId;
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

    public NftId getNftId() {
        return nftId;
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