// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

public class LedgerAddressGeneration extends WalletEvent {

    private String address;

    public String getAddress() {
        return address;
    }
}
