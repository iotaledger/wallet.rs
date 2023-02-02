// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.AddressWithAmount;
import org.iota.types.TransactionOptions;

/// Prepare send amount.
public class PrepareSendAmount implements AccountMethod {

    private AddressWithAmount[] addressesWithAmount;
    private TransactionOptions options;

    public PrepareSendAmount withAddressesWithAmount(AddressWithAmount[] addressesWithAmount) {
        this.addressesWithAmount = addressesWithAmount;
        return this;
    }

    public PrepareSendAmount withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}