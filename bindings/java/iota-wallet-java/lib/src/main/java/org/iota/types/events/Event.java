// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events;

import org.iota.types.AbstractObject;
import org.iota.types.events.wallet.WalletEvent;

public class Event extends AbstractObject {

    private Integer accountIndex;
    private WalletEvent event;

    public Integer getAccountIndex() {
        return accountIndex;
    }

    public WalletEvent getEvent() {
        return event;
    }
}
