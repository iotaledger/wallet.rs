// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;
public class ExpirationUnlockCondition extends UnlockCondition {

    private int type = 3;
    private Address returnAddress;
    private int unixTime;

    public int getType() {
        return type;
    }

    public Address getReturnAddress() {
        return returnAddress;
    }

    public ExpirationUnlockCondition withReturnAddress(Address returnAddress) {
        this.returnAddress = returnAddress;
        return this;
    }

    public int getUnixTime() {
        return unixTime;
    }

    public ExpirationUnlockCondition withUnixTime(int unixTime) {
        this.unixTime = unixTime;
        return this;
    }
}