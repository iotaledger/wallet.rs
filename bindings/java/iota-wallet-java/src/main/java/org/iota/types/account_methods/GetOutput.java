package org.iota.types.account_methods;

import org.iota.types.ids.OutputId;

/// Get the [`OutputData`] of an output stored in the account
public class GetOutput implements AccountMethod {

    private OutputId outputId;

    public GetOutput withOutputId(OutputId outputId) {
        this.outputId = outputId;
        return this;
    }
}