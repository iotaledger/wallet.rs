// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

public class RequiredStorageDeposit extends AbstractObject {

    private long alias;
    private long basic;
    private long foundry;
    private long nft;

    public long getAlias() {
        return alias;
    }

    public long getBasic() {
        return basic;
    }

    public long getFoundry() {
        return foundry;
    }

    public long getNft() {
        return nft;
    }
}