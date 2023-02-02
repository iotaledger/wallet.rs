// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.AliasId;

/// Destroy an alias output.
public class DestroyAlias implements AccountMethod {

    private AliasId aliasId;
    private TransactionOptions transactionOptions;

    public DestroyAlias withAliasId(AliasId aliasId) {
        this.aliasId = aliasId;
        return this;
    }

    public DestroyAlias withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}