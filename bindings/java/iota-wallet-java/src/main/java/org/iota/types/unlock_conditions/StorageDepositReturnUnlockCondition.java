package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;

public class StorageDepositReturnUnlockCondition extends UnlockCondition {
    private int type;
    private Address returnAddress;
    private String amount;
}