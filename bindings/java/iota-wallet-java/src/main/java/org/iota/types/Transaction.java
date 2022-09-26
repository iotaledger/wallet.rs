// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.BlockId;
import org.iota.types.ids.TransactionId;
import org.iota.types.payload.TransactionPayload;

public class Transaction extends AbstractObject {

    /// The transaction payload
    private TransactionPayload payload;
    /// BlockId when it got sent to the Tangle
    private BlockId blockId;
    /// Inclusion state of the transaction
    private InclusionState inclusionState;
    /// Timestamp
    private String timestamp;
    /// Transaction ID
    private TransactionId transactionId;
    /// Network id to ignore outputs when set_client_options is used to switch to another network
    private String networkId;
    /// If the transaction was created by the wallet or if it was sent by someone else and is incoming
    private boolean incoming;
    /// Note.
    private String note;

    public enum InclusionState {
        Pending,
        Confirmed,
        Conflicting,
        UnknownPruned,
    }


    public TransactionPayload getPayload() {
        return payload;
    }

    public BlockId getBlockId() {
        return blockId;
    }

    public InclusionState getInclusionState() {
        return inclusionState;
    }

    public String getTimestamp() {
        return timestamp;
    }

    public TransactionId getTransactionId() {
        return transactionId;
    }

    public String getNetworkId() {
        return networkId;
    }

    public boolean isIncoming() {
        return incoming;
    }

    public String getNote() {
        return note;
    }
}
