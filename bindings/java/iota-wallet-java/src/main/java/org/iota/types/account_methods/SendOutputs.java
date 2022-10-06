// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.outputs.Output;

/// Send outputs in a transaction.
public class SendOutputs implements AccountMethod {

    private Output[] outputs;
    private TransactionOptions options;

    public SendOutputs withOutputs(Output[] outputs) {
        this.outputs = outputs;
        return this;
    }

    public SendOutputs withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}