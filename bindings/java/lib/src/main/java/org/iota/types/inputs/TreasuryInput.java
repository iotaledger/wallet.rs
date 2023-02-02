// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.inputs;

import org.iota.types.ids.MilestoneId;
public class TreasuryInput extends Input {
    private int type = 1;
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
