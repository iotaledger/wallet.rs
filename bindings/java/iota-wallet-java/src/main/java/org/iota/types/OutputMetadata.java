// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.BlockId;
import org.iota.types.ids.TransactionId;

public class OutputMetadata {

    private BlockId blockId;
    private TransactionId transactionId;
    private int outputIndex;
    private boolean isSpent;
    private Integer milestoneIndexSpent;
    private Integer milestoneTimestampSpent;
    private TransactionId transactionIdSpent;
    private int milestoneIndexBooked;
    private int milestoneTimestampBooked;
    private int ledgerIndex;

    public BlockId getBlockId() {
        return blockId;
    }

    public TransactionId getTransactionId() {
        return transactionId;
    }

    public int getOutputIndex() {
        return outputIndex;
    }

    public boolean isSpent() {
        return isSpent;
    }

    public Integer getMilestoneIndexSpent() {
        return milestoneIndexSpent;
    }

    public Integer getMilestoneTimestampSpent() {
        return milestoneTimestampSpent;
    }

    public TransactionId getTransactionIdSpent() {
        return transactionIdSpent;
    }

    public int getMilestoneIndexBooked() {
        return milestoneIndexBooked;
    }

    public int getMilestoneTimestampBooked() {
        return milestoneTimestampBooked;
    }

    public int getLedgerIndex() {
        return ledgerIndex;
    }
}