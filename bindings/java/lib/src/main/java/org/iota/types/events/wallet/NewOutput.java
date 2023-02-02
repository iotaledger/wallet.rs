// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import org.iota.types.OutputData;
import org.iota.types.OutputResponse;
import org.iota.types.payload.TransactionPayload;

public class NewOutput extends WalletEvent {

    /// The new output.
    private OutputData output;

    /// The transaction that created the output. Might be pruned and not available.
    private TransactionPayload transaction;

    /// The inputs for the transaction that created the output. Might be pruned and
    /// not available.
    private OutputResponse[] transactionInputs;

    public NewOutput() {
        super(WalletEventType.NewOutput);
    }

    public OutputData getOutput() {
        return output;
    }

    public TransactionPayload getTransaction() {
        return transaction;
    }

    public OutputResponse[] getTransactionInputs() {
        return transactionInputs;
    }
}
