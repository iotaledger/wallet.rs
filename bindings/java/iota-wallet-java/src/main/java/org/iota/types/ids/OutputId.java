// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(OutputIdAdapter.class)
public class OutputId extends AbstractId {
    public OutputId(String id) {
        super(id);
    }
}

class OutputIdAdapter implements JsonDeserializer<OutputId> {
    @Override
    public OutputId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new OutputId(json.getAsString());
    }
}

