// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import org.iota.types.OutputData;

public class SpentOutput extends WalletEvent {

    private OutputData output;

    public SpentOutput(OutputData output) {
        super(WalletEventType.SpentOutput);
        this.output = output;
    }

    public OutputData getOutput() {
        return output;
    }
}
