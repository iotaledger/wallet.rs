package org.iota.types.account_methods;

import org.iota.types.AddressNativeTokens;
import org.iota.types.TransactionOptions;

/// Send native tokens.
public class SendNativeTokens implements AccountMethod {

    private AddressNativeTokens[] addressesNativeTokens;
    private TransactionOptions options;

    public SendNativeTokens withAddressesNativeTokens(AddressNativeTokens[] addressesNativeTokens) {
        this.addressesNativeTokens = addressesNativeTokens;
        return this;
    }

    public SendNativeTokens withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}