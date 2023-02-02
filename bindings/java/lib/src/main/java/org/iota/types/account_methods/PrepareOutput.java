// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.OutputOptions;
import org.iota.types.TransactionOptions;

/// Prepare an output.
public class PrepareOutput implements AccountMethod {

    private OutputOptions options;
    private TransactionOptions transactionOptions;

    public PrepareOutput withOptions(OutputOptions options) {
        this.options = options;
        return this;
    }

    public PrepareOutput withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}

