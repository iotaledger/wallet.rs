// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.TransactionId;

/// Get the transaction with inputs of an incoming transaction stored in the account
/// List might not be complete, if the node pruned the data already
public class GetIncomingTransactionData implements AccountMethod {

    private TransactionId transactionId;

    public GetIncomingTransactionData withTransactionId(TransactionId transactionId) {
        this.transactionId = transactionId;
        return this;
    }
}