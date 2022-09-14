package org.iota.types;

public class AccountAddress {

    /// The address.
    private String address;
    /// The address key index.
    private int keyIndex;
    /// Determines if an address is a public or an internal (change) address.
    private boolean isInternal;
    // do we want this field? Could be useful if we don't store spent output ids and because of that wouldn't know if
    // an address was used or not just by looking at it
    private boolean used;
}
