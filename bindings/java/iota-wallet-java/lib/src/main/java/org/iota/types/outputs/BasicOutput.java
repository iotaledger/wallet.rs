// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.outputs;

import org.iota.types.NativeToken;
import org.iota.types.features.Feature;
import org.iota.types.unlock_conditions.UnlockCondition;
public class BasicOutput extends Output {

        private int type = 3;
        // Amount of IOTA tokens held by the output.
        private String amount;
        // Native tokens held by the output.
        private NativeToken[] nativeTokens;
        // Native tokens held by the output.
        private UnlockCondition[] unlockConditions;
        private Feature[] features;

        public BasicOutput(String amount, NativeToken[] nativeTokens, UnlockCondition[] unlockConditions, Feature[] features) {
                this.amount = amount;
                this.nativeTokens = nativeTokens;
                this.unlockConditions = unlockConditions;
                this.features = features;
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

        public UnlockCondition[] getUnlockConditions() {
                return unlockConditions;
        }

        public Feature[] getFeatures() {
                return features;
        }
}
