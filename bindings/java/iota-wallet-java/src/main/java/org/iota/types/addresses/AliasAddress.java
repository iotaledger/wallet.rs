package org.iota.types.addresses;

import org.iota.types.addresses.Address;
import org.iota.types.ids.AliasId;

public class AliasAddress extends Address {
    private AliasId aliasId;

    public AliasAddress(AliasId aliasId) {
        this.aliasId = aliasId;
    }

    public AliasId getAliasId() {
        return aliasId;
    }

    @Override
    public String toString() {
        return aliasId.toString();
    }
}
