package org.iota.types;

import org.iota.types.addresses.Address;

public class AddressWrapper extends AbstractObject {

    /// The account index.
    private Address inner;
    /// The bech32 HRP.
    private String bech32Hrp;

}