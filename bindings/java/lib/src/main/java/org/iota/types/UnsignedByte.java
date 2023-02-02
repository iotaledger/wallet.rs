// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(UnsignedByteAdapter.class)
public class UnsignedByte {
    private byte signedByte;

    public UnsignedByte(byte signedByte) {
        this.signedByte = signedByte;
    }

    public byte getAsSignedByte() {
        return signedByte;
    }
}

class UnsignedByteAdapter implements JsonSerializer<UnsignedByte>, JsonDeserializer<UnsignedByte> {
    @Override
    public JsonElement serialize(UnsignedByte src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.getAsSignedByte() & 0xFF);
    }

    @Override
    public UnsignedByte deserialize(JsonElement json, Type typeOfT, JsonDeserializationContext context) throws JsonParseException {
        return new UnsignedByte(json.getAsByte());
    }
}
