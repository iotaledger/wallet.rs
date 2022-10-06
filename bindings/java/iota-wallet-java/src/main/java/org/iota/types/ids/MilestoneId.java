// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(MilestoneIdAdapter.class)
public class MilestoneId extends AbstractId {
    public MilestoneId(String id) {
        super(id);
    }
}

class MilestoneIdAdapter implements JsonDeserializer<MilestoneId>, JsonSerializer<MilestoneId> {
    @Override
    public MilestoneId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new MilestoneId(json.getAsString());
    }
    @Override
    public JsonElement serialize(MilestoneId src, Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
    }
}

