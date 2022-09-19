package org.iota.types.inputs;

import org.iota.types.ids.MilestoneId;

public class TreasuryInput extends Input {
    private int type;
    private MilestoneId transactionId;

    public TreasuryInput(MilestoneId transactionId) {
        this.transactionId = transactionId;
    }

    public int getType() {
        return type;
    }

    public MilestoneId getTransactionId() {
        return transactionId;
    }
}
