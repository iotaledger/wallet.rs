// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.TransactionId;

/// Get the [`Transaction`] of a transaction stored in the account
public class GetTransaction implements AccountMethod {

    private TransactionId transactionId;

    public GetTransaction withTransactionId(TransactionId transactionId) {
        this.transactionId = transactionId;
        return this;
    }
}