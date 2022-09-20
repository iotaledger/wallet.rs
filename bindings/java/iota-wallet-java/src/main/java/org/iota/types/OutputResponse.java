// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.outputs.Output;

public class OutputResponse {

    /// The actual Output.
    private Output output;
    /// The metadata of the output.
    private OutputMetadata metadata;

    public Output getOutput() {
        return output;
    }

    public OutputMetadata getMetadata() {
        return metadata;
    }
}

