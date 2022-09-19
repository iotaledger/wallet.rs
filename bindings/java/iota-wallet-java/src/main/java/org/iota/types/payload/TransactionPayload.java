// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.payload;

import org.iota.types.transaction_essence.TransactionEssence;
import org.iota.types.unlocks.Unlock;

public class TransactionPayload extends Payload {

    private int type;
    private TransactionEssence essence;
    private Unlock[] unlocks;

}