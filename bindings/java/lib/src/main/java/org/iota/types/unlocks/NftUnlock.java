// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlocks;
public class NftUnlock extends Unlock {
    private int type = 3;
    private int reference;

    public NftUnlock(int reference) {
        this.reference = reference;
    }

    public int getType() {
        return type;
    }

    public int getReference() {
        return reference;
    }

}