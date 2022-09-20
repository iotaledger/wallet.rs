// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.ids;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(BlockIdAdapter.class)
public class BlockId extends AbstractId {
    public BlockId(String id) {
        super(id);
    }
}

class BlockIdAdapter implements JsonDeserializer<BlockId> {
    @Override
    public BlockId deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {
        return new BlockId(json.getAsString());
    }
}

