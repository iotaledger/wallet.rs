// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(FoundryIdAdapter.class)
public class FoundryId extends AbstractId {
    public FoundryId(String id) {
        super(id);
    }
}

class FoundryIdAdapter implements JsonDeserializer<FoundryId> {
    @Override
    public FoundryId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new FoundryId(json.getAsString());
    }
}

