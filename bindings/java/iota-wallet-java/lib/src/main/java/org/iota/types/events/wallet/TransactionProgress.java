// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import org.iota.types.events.transactionprogress.TransactionProgressEvent;

public class TransactionProgress extends WalletEvent {

    private TransactionProgressEvent event;

    public TransactionProgress(TransactionProgressEvent event) {
        this.event = event;
    }

    public TransactionProgressEvent getEvent() {
        return event;
    }
}
