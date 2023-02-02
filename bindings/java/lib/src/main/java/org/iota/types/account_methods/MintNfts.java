// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.NftOptions;
import org.iota.types.TransactionOptions;

/// Mint nft.
public class MintNfts implements AccountMethod {

    private NftOptions[] nftsOptions;
    private TransactionOptions options;

    public MintNfts withNftsOptions(NftOptions[] nftsOptions) {
        this.nftsOptions = nftsOptions;
        return this;
    }

    public MintNfts withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}