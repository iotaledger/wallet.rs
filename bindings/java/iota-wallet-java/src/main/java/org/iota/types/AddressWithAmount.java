package org.iota.types;

public class AddressWithAmount extends AbstractObject {
    /// Bech32 encoded address
    private String address;
    /// Amount
    private String amount;

    public String getAddress() {
        return address;
    }

    public AddressWithAmount withAddress(String address) {
        this.address = address;
        return this;
    }

    public String getAmount() {
        return amount;
    }

    public AddressWithAmount withAmount(String amount) {
        this.amount = amount;
        return this;
    }
}
