// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import org.iota.types.Transaction.InclusionState;
import org.iota.types.ids.TransactionId;

public class TransactionInclusion extends WalletEvent {

    TransactionId transactionId;

    InclusionState inclusionState;

    public TransactionInclusion(TransactionId transactionId, InclusionState inclusionState) {
        super(WalletEventType.TransactionInclusion);
        this.transactionId = transactionId;
        this.inclusionState = inclusionState;
    }

    public TransactionId getTransactionId() {
        return transactionId;
    }

    public InclusionState getInclusionState() {
        return inclusionState;
    }
}
