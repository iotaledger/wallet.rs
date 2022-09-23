// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlocks;
public class ReferenceUnlock extends Unlock {
    private int type = 1;
    private int reference;

    public ReferenceUnlock(int reference) {
        this.reference = reference;
    }

    public int getType() {
        return type;
    }

    public int getReference() {
        return reference;
    }

}