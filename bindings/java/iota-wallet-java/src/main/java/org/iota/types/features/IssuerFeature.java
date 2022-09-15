package org.iota.types.features;

import org.iota.types.addresses.Address;

public class IssuerFeature extends Feature {

    private int type = 1;
    private Address address;

    public int getType() {
        return type;
    }

    public Address getAddress() {
        return address;
    }

    public IssuerFeature withAddress(Address address) {
        this.address = address;
        return this;
    }
}