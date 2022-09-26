// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.addresses;

import org.iota.types.ids.NftId;
public class NftAddress extends Address {

    private int type = 16;
    private NftId nftId;

    public NftAddress(NftId nftId) {
        this.nftId = nftId;
    }

    public int getType() {
        return type;
    }

    public NftId getNftId() {
        return nftId;
    }

    @Override
    public String toString() {
        return nftId.toString();
    }
}
