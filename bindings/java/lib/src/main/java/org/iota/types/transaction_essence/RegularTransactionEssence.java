// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.transaction_essence;

import org.iota.types.inputs.Input;
import org.iota.types.outputs.Output;
import org.iota.types.payload.TaggedDataPayload;
public class RegularTransactionEssence extends TransactionEssence {
    private int type = 1;
    private String networkId;
    private Input[] inputs;
    private String inputsCommitment;
    private Output[] outputs;
    private TaggedDataPayload payload;

    public RegularTransactionEssence(String networkId, Input[] inputs, String inputsCommitment, Output[] outputs, TaggedDataPayload payload) {
        this.networkId = networkId;
        this.inputs = inputs;
        this.inputsCommitment = inputsCommitment;
        this.outputs = outputs;
        this.payload = payload;
    }

    public int getType() {
        return type;
    }

    public String getNetworkId() {
        return networkId;
    }

    public Input[] getInputs() {
        return inputs;
    }

    public String getInputsCommitment() {
        return inputsCommitment;
    }

    public Output[] getOutputs() {
        return outputs;
    }

    public TaggedDataPayload getPayload() {
        return payload;
    }
}