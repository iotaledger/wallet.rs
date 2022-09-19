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

}