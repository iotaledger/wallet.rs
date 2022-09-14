package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;

public class ExpirationUnlockCondition extends UnlockCondition {
    private int type;
    private Address returnAddress;
    private int unixTime;
}