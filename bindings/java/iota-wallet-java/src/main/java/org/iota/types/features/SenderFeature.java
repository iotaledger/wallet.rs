// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.features;

import org.iota.types.addresses.Address;
public class SenderFeature extends Feature {

    private int type = 0;
    private Address address;

    public int getType() {
        return type;
    }

    public Address getAddress() {
        return address;
    }

    public SenderFeature withAddress(Address address) {
        this.address = address;
        return this;
    }
}