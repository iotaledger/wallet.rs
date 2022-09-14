package org.iota.types;

import org.iota.types.addresses.Address;

public class Ed25519Address extends Address {
    private String address;

    public Ed25519Address(String address) {
        this.address = address;
    }

    @Override
    public String toString() {
        return address;
    }
}
