// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.NftId;
public class AddressAndNftId extends AbstractObject {

    /// Bech32 encoded address
    private String address;
    /// Nft id
    private NftId nftId;

    public String getAddress() {
        return address;
    }

    public AddressAndNftId withAddress(String address) {
        this.address = address;
        return this;
    }

    public NftId getNftId() {
        return nftId;
    }

    public AddressAndNftId withNftId(NftId nftId) {
        this.nftId = nftId;
        return this;
    }
}