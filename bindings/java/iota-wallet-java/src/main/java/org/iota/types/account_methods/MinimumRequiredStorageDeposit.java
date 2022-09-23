// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.outputs.Output;

/// Calculate the minimum required storage deposit for an output.
public class MinimumRequiredStorageDeposit implements AccountMethod {

    private Output output;

    public MinimumRequiredStorageDeposit withOutput(Output output) {
        this.output = output;
        return this;
    }
}