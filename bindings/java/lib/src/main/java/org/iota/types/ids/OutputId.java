// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(OutputIdAdapter.class)
public class OutputId extends AbstractId {
    public OutputId(String id) {
        super(id);
    }
}

class OutputIdAdapter implements JsonDeserializer<OutputId>, JsonSerializer<OutputId> {
    @Override
    public OutputId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new OutputId(json.getAsString());
    }
    @Override
    public JsonElement serialize(OutputId src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
    }
}

