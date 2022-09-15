package org.iota.types;

public class NftOptions extends AbstractObject {
    /// Bech32 encoded address to which the Nft will be minted. Default will use the first address of the account
    private String address;
    /// Immutable nft metadata, hex encoded bytes
    private String immutableMetadata;
    /// Nft metadata, hex encoded bytes
    private String metadata;
}