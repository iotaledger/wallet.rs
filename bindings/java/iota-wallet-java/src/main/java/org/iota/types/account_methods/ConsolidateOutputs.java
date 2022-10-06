// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

/// Consolidate outputs.
public class ConsolidateOutputs implements AccountMethod {

    private boolean force;
    private Integer outputConsolidationThreshold;

    public ConsolidateOutputs withForce(boolean force) {
        this.force = force;
        return this;
    }

    public ConsolidateOutputs withOutputConsolidationThreshold(Integer outputConsolidationThreshold) {
        this.outputConsolidationThreshold = outputConsolidationThreshold;
        return this;
    }
}