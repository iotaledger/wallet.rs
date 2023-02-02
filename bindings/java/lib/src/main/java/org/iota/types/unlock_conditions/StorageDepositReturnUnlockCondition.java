// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;

import org.iota.types.addresses.Address;
public class StorageDepositReturnUnlockCondition extends UnlockCondition {

    private int type = 1;
    private Address returnAddress;
    private String amount;

    public int getType() {
        return type;
    }

    public Address getReturnAddress() {
        return returnAddress;
    }

    public StorageDepositReturnUnlockCondition withReturnAddress(Address returnAddress) {
        this.returnAddress = returnAddress;
        return this;
    }

    public String getAmount() {
        return amount;
    }

    public StorageDepositReturnUnlockCondition withAmount(String amount) {
        this.amount = amount;
        return this;
    }
}