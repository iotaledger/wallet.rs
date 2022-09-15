package org.iota.types.addresses;

import org.iota.types.addresses.Address;
import org.iota.types.ids.AliasId;

public class AliasAddress extends Address {

    private int type = 8;
    private AliasId aliasId;

    public AliasAddress(AliasId aliasId) {
        this.aliasId = aliasId;
    }

    public int getType() {
        return type;
    }

    public AliasId getAliasId() {
        return aliasId;
    }

    @Override
    public String toString() {
        return aliasId.toString();
    }
}
