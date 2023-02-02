// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transaction;

public class SigningTransaction extends TransactionProgressEvent {
    public SigningTransaction() {
        super(TransactionProgressEventType.SigningTransaction);
    }
}
