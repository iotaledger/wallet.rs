// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlocks;
public class AliasUnlock extends Unlock {
    private int type = 2;
    private int reference;

    public AliasUnlock(int reference) {
        this.reference = reference;
    }

    public int getType() {
        return type;
    }

    public int getReference() {
        return reference;
    }

}