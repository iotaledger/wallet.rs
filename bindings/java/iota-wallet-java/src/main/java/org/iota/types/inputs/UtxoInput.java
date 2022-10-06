// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.inputs;

import org.iota.types.ids.TransactionId;
public class UtxoInput extends Input {
     int type = 0;
     TransactionId transactionId;
     int transactionOutputIndex;

    public UtxoInput(TransactionId transactionId, int transactionOutputIndex) {
        this.transactionId = transactionId;
        this.transactionOutputIndex = transactionOutputIndex;
    }

    public int getType() {
        return type;
    }

    public TransactionId getTransactionId() {
        return transactionId;
    }

    public int getTransactionOutputIndex() {
        return transactionOutputIndex;
    }
}
