// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

public class ConsolidationRequired extends WalletEvent {
    public ConsolidationRequired() {
        super(WalletEventType.ConsolidationRequired);
    }
}
