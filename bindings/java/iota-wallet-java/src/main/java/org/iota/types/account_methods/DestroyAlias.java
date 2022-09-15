package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.AliasId;

/// Destroy an alias output. Outputs controlled by it will be sweeped before if they don't have a
/// storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
/// sent to the governor address.
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