// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.PreparedTransactionData;

/// Sign a prepared transaction.
public class SignTransactionEssence implements AccountMethod {

    private PreparedTransactionData preparedTransactionData;

    public SignTransactionEssence withPreparedTransactionData(PreparedTransactionData preparedTransactionData) {
        this.preparedTransactionData = preparedTransactionData;
        return this;
    }

    public PreparedTransactionData getPreparedTransactionData() {
        return preparedTransactionData;
    }
}