// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transactionprogress;

public class GeneratingRemainderDepositAddress extends TransactionProgressEvent {

    private String address;

    public GeneratingRemainderDepositAddress(String address) {
        this.address = address;
    }

    public String getAddress() {
        return address;
    }
}
