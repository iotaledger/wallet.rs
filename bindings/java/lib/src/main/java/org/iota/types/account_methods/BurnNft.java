// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.NftId;

/// Burn a nft output. Outputs controlled by it will be sweeped before if they don't have a storage
/// deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
/// burning, the foundry can never be destroyed anymore.
public class BurnNft implements AccountMethod {

    private NftId nftId;
    private TransactionOptions transactionOptions;

    public BurnNft withNftId(NftId nftId) {
        this.nftId = nftId;
        return this;
    }

    public BurnNft withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}