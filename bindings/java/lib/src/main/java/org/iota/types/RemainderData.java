// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.addresses.Address;
import org.iota.types.outputs.Output;

public class RemainderData extends AbstractObject {
    /// The remainder output
    private Output output;
    /// The chain derived from seed, for the remainder addresses
    private Segment[] chain;
    /// The remainder address
    private Address address;
}