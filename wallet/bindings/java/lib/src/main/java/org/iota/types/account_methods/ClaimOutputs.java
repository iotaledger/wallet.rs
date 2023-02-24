// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.OutputId;

/// Claim outputs.
public class ClaimOutputs implements AccountMethod {

    private OutputId[] outputIdsToClaim;

    public ClaimOutputs withOutputIdsToClaim(OutputId[] outputIdsToClaim) {
        this.outputIdsToClaim = outputIdsToClaim;
        return this;
    }
}