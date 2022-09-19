package org.iota.types.inputs;

import org.iota.types.ids.TransactionId;

public class UtxoInput extends Input {
    private int type;
    private TransactionId transactionId;
    private int transactionOutputIndex;

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
