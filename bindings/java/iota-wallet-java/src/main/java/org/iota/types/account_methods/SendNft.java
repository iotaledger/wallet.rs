package org.iota.types.account_methods;

import org.iota.types.AddressAndNftId;
import org.iota.types.TransactionOptions;

/// Send nft.
public class SendNft implements AccountMethod {

    private AddressAndNftId[] addressesAndNftIds;
    private TransactionOptions options;

    public SendNft withAddressesAndNftIds(AddressAndNftId[] addressesAndNftIds) {
        this.addressesAndNftIds = addressesAndNftIds;
        return this;
    }

    public SendNft withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}