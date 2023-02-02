// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.AddressWithAmount;
import org.iota.types.TransactionOptions;

/// Send amount.
public class SendAmount implements AccountMethod {

    private AddressWithAmount[] addressesWithAmount;
    private TransactionOptions options;

    public SendAmount withAddressesWithAmount(AddressWithAmount[] addressesWithAmount) {
        this.addressesWithAmount = addressesWithAmount;
        return this;
    }

    public SendAmount withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}