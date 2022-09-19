// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.transaction_essence.TransactionEssence;

public class PreparedTransactionData extends AbstractObject {

    private TransactionEssence essence;
    private InputSigningData inputsData;
    private RemainderData remainderData;

    public PreparedTransactionData(TransactionEssence essence, InputSigningData inputsData, RemainderData remainderData) {
        this.essence = essence;
        this.inputsData = inputsData;
        this.remainderData = remainderData;
    }

    public TransactionEssence getEssence() {
        return essence;
    }

    public InputSigningData getInputsData() {
        return inputsData;
    }

    public RemainderData getRemainderData() {
        return remainderData;
    }
}