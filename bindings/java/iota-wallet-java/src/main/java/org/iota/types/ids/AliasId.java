// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(AliasIdAdapter.class)
public class AliasId extends AbstractId {
    public AliasId(String id) {
        super(id);
    }
}

class AliasIdAdapter implements JsonDeserializer<AliasId>, JsonSerializer<AliasId> {
    @Override
    public AliasId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new AliasId(json.getAsString());
    }
    @Override
    public JsonElement serialize(AliasId src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
    }
}

