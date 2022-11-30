// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.TransactionId;

/// Retry a transaction until it's included.
public class RetryTransactionUntilIncluded implements AccountMethod {

    private TransactionId transaction_id;
    private Int interval;
    private Int maxAttempts;

    public RetryTransactionUntilIncluded withTransactionId(TransactionId transaction_id) {
        this.transaction_id = transaction_id;
        return this;
    }

    public RetryTransactionUntilIncluded withInterval(Int interval) {
        this.interval = interval;
        return this;
    }

    public RetryTransactionUntilIncluded withMaxAttempts(Int maxAttempts) {
        this.maxAttempts = maxAttempts;
        return this;
    }
}