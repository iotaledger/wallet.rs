// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transaction;

public class Broadcasting extends TransactionProgressEvent {

    public Broadcasting() {
        super(TransactionProgressEventType.Broadcasting);
    }
}
