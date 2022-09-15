package org.iota.types.account_methods;

import org.iota.types.NftOptions;
import org.iota.types.TransactionOptions;

/// Mint nft.
public class MintNfts implements AccountMethod {

    private NftOptions[] nftOptions;
    private TransactionOptions options;

    public MintNfts withNftOptions(NftOptions[] nftOptions) {
        this.nftOptions = nftOptions;
        return this;
    }

    public MintNfts withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}