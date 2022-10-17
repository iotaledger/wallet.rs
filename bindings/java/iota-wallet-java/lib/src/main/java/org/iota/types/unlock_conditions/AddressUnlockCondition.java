// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;
public class AddressUnlockCondition extends UnlockCondition {

    private int type = 0;
    private Address address;

    public int getType() {
        return type;
    }

    public Address getAddress() {
        return address;
    }

    public AddressUnlockCondition withAddress(Address address) {
        this.address = address;
        return this;
    }
}