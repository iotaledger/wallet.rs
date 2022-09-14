package org.iota.types;

import org.iota.types.ids.OutputId;

public class AddressWithUnspentOutputs {

    /// The address.
    private AddressWrapper address;
    /// The address key index.
    private int keyIndex;
    /// Determines if an address is a public or an internal (change) address.
    private boolean internal;
    /// Output ids.
    private OutputId[] output_ids;

}


