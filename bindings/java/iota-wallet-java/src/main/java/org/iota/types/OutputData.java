// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.addresses.Address;
import org.iota.types.ids.OutputId;
import org.iota.types.outputs.Output;

public class OutputData {

    /// The output id.
    private OutputId outputId;
    /// The metadata of the output.
    private OutputMetadata metadata;
    /// The actual Output.
    private Output output;
    /// If an output is spent.
    private boolean isSpent;
    /// Associated account address.
    private Address address;
    /// Network ID.
    private String networkId;
    /// Remainder.
    private boolean remainder;
    /// Bip32 path.
    private Segment[] chain;

}

