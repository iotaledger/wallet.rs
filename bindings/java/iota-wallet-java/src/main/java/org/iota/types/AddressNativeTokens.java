// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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

    public String getAddress() {
        return address;
    }

    public AddressNativeTokens withAddress(String address) {
        this.address = address;
        return this;
    }

    public Map<TokenId, String>[] getNativeTokens() {
        return nativeTokens;
    }

    public AddressNativeTokens withNativeTokens(Map<TokenId, String>[] nativeTokens) {
        this.nativeTokens = nativeTokens;
        return this;
    }

    public String getReturnAddress() {
        return returnAddress;
    }

    public AddressNativeTokens withReturnAddress(String returnAddress) {
        this.returnAddress = returnAddress;
        return this;
    }

    public int getExpiration() {
        return expiration;
    }

    public AddressNativeTokens withExpiration(int expiration) {
        this.expiration = expiration;
        return this;
    }
}

