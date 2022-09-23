// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.unlock_conditions.UnlockCondition;

/// Build a BasicOutput.
public class BuildBasicOutput implements AccountMethod {

    // If not provided, minimum storage deposit will be used
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
}