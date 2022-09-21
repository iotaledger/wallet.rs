// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(TransactionIdAdapter.class)
public class TransactionId extends AbstractId {
    public TransactionId(String id) {
        super(id);
    }
}

class TransactionIdAdapter implements JsonDeserializer<TransactionId>, JsonSerializer<TransactionId> {
    @Override
    public TransactionId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new TransactionId(json.getAsString());
    }
    @Override
    public JsonElement serialize(TransactionId src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
    }
}

