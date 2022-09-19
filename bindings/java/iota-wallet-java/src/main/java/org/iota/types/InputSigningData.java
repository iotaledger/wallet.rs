package org.iota.types;

import org.iota.types.outputs.Output;

public class InputSigningData extends AbstractObject {
    /// The output
    private Output output;
    /// The output metadata
    private OutputMetadata outputMetadata;
    /// The chain derived from seed, only for ed25519 addresses
    /// Bip32 path.
    private Segment[] chain;
    /// The bech32 encoded address, required because of alias outputs where we have multiple possible unlock
    /// conditions, because we otherwise don't know which one we need
    private String bech32Address;
}