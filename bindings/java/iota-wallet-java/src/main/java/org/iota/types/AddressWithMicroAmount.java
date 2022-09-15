package org.iota.types;

public class AddressWithMicroAmount extends AbstractObject {
    /// Bech32 encoded address
    private String address;
    /// Amount below the minimum storage deposit
    private String amount;
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    private String returnAddress;
    /// Expiration in seconds, after which the output will be available for the sender again, if not spent by the
    /// receiver before. Default is 1 day
    private int expiration;
}