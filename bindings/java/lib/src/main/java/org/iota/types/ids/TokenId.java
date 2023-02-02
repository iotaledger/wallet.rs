// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(TokenIdAdapter.class)
public class TokenId extends AbstractId {
    public TokenId(String id) {
        super(id);
    }
}

class TokenIdAdapter implements JsonDeserializer<TokenId>, JsonSerializer<TokenId> {
    @Override
    public TokenId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new TokenId(json.getAsString());
    }

    @Override
    public JsonElement serialize(TokenId src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
    }
}

