// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;
public class StateControllerAddressUnlockCondition extends UnlockCondition {

    private int type = 4;
    private Address address;

    public int getType() {
        return type;
    }

    public Address getAddress() {
        return address;
    }

    public StateControllerAddressUnlockCondition withAddress(Address address) {
        this.address = address;
        return this;
    }
}