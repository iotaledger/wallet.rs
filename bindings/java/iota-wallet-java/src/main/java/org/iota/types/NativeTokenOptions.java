package org.iota.types;

public class NativeTokenOptions extends NewAbstractObject {
    /// Bech32 encoded address. Needs to be an account address. Default will use the first address of the account
    private String accountAddress;
    /// Circulating supply
    private String circulatingSupply;
    /// Maximum supply
    private String maximumSupply;
    /// Foundry metadata, hex encoded bytes
    private String foundryMetadata;
}