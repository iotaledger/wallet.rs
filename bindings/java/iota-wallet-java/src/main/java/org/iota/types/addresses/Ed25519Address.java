// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.addresses;
public class Ed25519Address extends Address {

    private int type = 0;
    private String pubKeyHash;

    public Ed25519Address(String pubKeyHash) {
        this.pubKeyHash = pubKeyHash;
    }

    public int getType() {
        return type;
    }

    public String getPubKeyHash() {
        return pubKeyHash;
    }

}
