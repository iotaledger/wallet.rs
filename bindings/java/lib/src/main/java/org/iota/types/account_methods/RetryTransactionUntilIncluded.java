// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.TransactionId;

/// Retry a transaction until it's included.
public class RetryTransactionUntilIncluded implements AccountMethod {

    private TransactionId transaction_id;
    private int interval;
    private int maxAttempts;

    public RetryTransactionUntilIncluded withTransactionId(TransactionId transaction_id) {
        this.transaction_id = transaction_id;
        return this;
    }

    public RetryTransactionUntilIncluded withInterval(int interval) {
        this.interval = interval;
        return this;
    }

    public RetryTransactionUntilIncluded withMaxAttempts(int maxAttempts) {
        this.maxAttempts = maxAttempts;
        return this;
    }
}