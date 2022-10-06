// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.outputs.Output;

/// Prepare transaction.
public class PrepareTransaction implements AccountMethod {

    private Output[] outputs;
    private TransactionOptions options;

    public PrepareTransaction withOutputs(Output[] outputs) {
        this.outputs = outputs;
        return this;
    }

    public PrepareTransaction withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}