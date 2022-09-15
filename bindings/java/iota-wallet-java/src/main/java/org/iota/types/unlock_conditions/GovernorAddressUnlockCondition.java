package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;

public class GovernorAddressUnlockCondition extends UnlockCondition {

    private int type = 5;
    private Address address;

    public int getType() {
        return type;
    }

    public Address getAddress() {
        return address;
    }

    public GovernorAddressUnlockCondition withAddress(Address address) {
        this.address = address;
        return this;
    }
}