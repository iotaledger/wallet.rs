// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transactionprogress;

public class PreparedTransactionEssenceHash extends TransactionProgressEvent {

    private String hash;

    public PreparedTransactionEssenceHash(String hash) {
        this.hash = hash;
    }

    public String getHash() {
        return hash;
    }
}
