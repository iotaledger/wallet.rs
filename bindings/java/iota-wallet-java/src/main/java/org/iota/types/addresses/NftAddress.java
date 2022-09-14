package org.iota.types.addresses;

import org.iota.types.addresses.Address;
import org.iota.types.ids.NftId;

public class NftAddress extends Address {
    private NftId nftId;

    public NftAddress(NftId nftId) {
        this.nftId = nftId;
    }

    public NftId getNftId() {
        return nftId;
    }

    @Override
    public String toString() {
        return nftId.toString();
    }
}
