package org.iota.types;

import org.iota.types.ids.TokenId;

import java.util.Map;

public class AddressNativeTokens extends AbstractObject {

    /// Bech32 encoded address
    private String address;
    /// Native tokens
    private Map<TokenId, String>[] nativeTokens;
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    private String returnAddress;
    /// Expiration in seconds, after which the output will be available for the sender again, if not spent by the
    /// receiver before. Default is 1 day
    private int expiration;
}

