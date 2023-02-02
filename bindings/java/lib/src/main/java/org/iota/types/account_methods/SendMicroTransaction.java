// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.AddressWithMicroAmount;
import org.iota.types.TransactionOptions;

/// Send amount below minimum storage deposit.
public class SendMicroTransaction implements AccountMethod {

    private AddressWithMicroAmount[] addressesWithMicroAmount;
    private TransactionOptions options;

    public SendMicroTransaction withAddressesWithMicroAmount(AddressWithMicroAmount[] addressesWithMicroAmount) {
        this.addressesWithMicroAmount = addressesWithMicroAmount;
        return this;
    }

    public SendMicroTransaction withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}