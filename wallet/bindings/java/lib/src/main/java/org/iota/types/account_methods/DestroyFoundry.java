// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.FoundryId;

/// Function to destroy a foundry output with a circulating supply of 0.
/// Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias
public class DestroyFoundry implements AccountMethod {

    private FoundryId foundryId;
    private TransactionOptions transactionOptions;

    public DestroyFoundry withFoundryId(FoundryId foundryId) {
        this.foundryId = foundryId;
        return this;
    }

    public DestroyFoundry withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}