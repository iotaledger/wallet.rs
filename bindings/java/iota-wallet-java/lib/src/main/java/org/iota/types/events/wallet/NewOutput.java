// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import java.util.Optional;

import org.iota.types.OutputData;
import org.iota.types.payload.TransactionPayload;

public class NewOutput extends WalletEvent {
    /// The new output.
    OutputData output;

    /// The transaction that created the output. Might be pruned and not available.
    Optional<TransactionPayload> transaction;

    /// The inputs for the transaction that created the output. Might be pruned and
    /// not available.
    // Optional<[OutputWithMetadataResponse]> transactionInputs;
}
