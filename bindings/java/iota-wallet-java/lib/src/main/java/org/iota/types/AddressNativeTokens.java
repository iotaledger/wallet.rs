// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;
import org.iota.api.CustomGson;
import org.iota.types.ids.TokenId;

import java.lang.reflect.Type;

public class AddressNativeTokens extends AbstractObject {

    /// Bech32 encoded address
    private String address;
    /// Native tokens
    private NativeTokenTuple[] nativeTokens;
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

    public NativeTokenTuple[] getNativeTokens() {
        return nativeTokens;
    }

    public AddressNativeTokens withNativeTokens(NativeTokenTuple[] nativeTokens) {
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

    @JsonAdapter(NativeTokenTupleAdapter.class)
    public static class NativeTokenTuple extends AbstractTuple {

        public NativeTokenTuple(TokenId tokenId, String amount) {
            super(tokenId, amount);
        }

        public TokenId getTokenId() {
            return (TokenId) get(0);
        }

        public String getAmount() {
            return (String) get(1);
        }

    }

    class NativeTokenTupleAdapter implements JsonSerializer<NativeTokenTuple> {
        @Override
        public JsonElement serialize(NativeTokenTuple t, Type typeOfSrc, JsonSerializationContext context) {
            JsonArray a = new JsonArray();
            a.add(CustomGson.get().toJsonTree(t.getTokenId()));
            a.add(t.getAmount());
            return a;
        }
    }

}

