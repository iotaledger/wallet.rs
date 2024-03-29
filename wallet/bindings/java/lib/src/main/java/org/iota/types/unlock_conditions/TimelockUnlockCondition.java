// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.unlock_conditions;
public class TimelockUnlockCondition extends UnlockCondition {

    private int type = 2;
    private int unixTime;

    public int getType() {
        return type;
    }

    public int getUnixTime() {
        return unixTime;
    }

    public TimelockUnlockCondition withUnixTime(int unixTime) {
        this.unixTime = unixTime;
        return this;
    }
}