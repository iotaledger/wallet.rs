package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;

public class ImmutableAliasAddressUnlockConditionDto extends UnlockCondition {
    private int type;
    private Address address;
}