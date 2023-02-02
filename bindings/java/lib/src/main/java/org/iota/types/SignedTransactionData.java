// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.payload.TransactionPayload;

public class SignedTransactionData extends AbstractObject {

    /// Transaction essence
    private TransactionPayload transactionPayload;
    /// Required address information for signing
    private InputSigningData[] inputsData;

    public TransactionPayload getTransactionPayload() {
        return transactionPayload;
    }

    public InputSigningData[] getInputsData() {
        return inputsData;
    }
}