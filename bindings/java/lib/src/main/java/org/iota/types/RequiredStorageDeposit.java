// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

public class RequiredStorageDeposit extends AbstractObject {

    private String alias;
    private String basic;
    private String foundry;
    private String nft;

    public String getAlias() {
        return alias;
    }

    public String getBasic() {
        return basic;
    }

    public String getFoundry() {
        return foundry;
    }

    public String getNft() {
        return nft;
    }
}