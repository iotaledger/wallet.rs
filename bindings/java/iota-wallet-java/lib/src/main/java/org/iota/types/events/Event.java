// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events;

import org.iota.types.AbstractObject;
import org.iota.types.events.wallet.WalletEvent;

public class Event extends AbstractObject {

    private Integer accountIndex;
    private WalletEvent event;

    public Event(Integer accountIndex, WalletEvent event) {
        this.accountIndex = accountIndex;
        this.event = event;
    }

    /**
     * Get the account index related to this event
     * 
     * @return The account index
     */
    public Integer getAccountIndex() {
        return accountIndex;
    }

    /**
     * Get the wallet event 
     * 
     * @return the event
     */
    public WalletEvent getEvent() {
        return event;
    }
}
