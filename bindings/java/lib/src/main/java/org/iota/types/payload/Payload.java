// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.payload;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;

import java.lang.reflect.Type;
@JsonAdapter(PayloadAdapter.class)
public abstract class Payload extends AbstractObject {}

class PayloadAdapter implements JsonDeserializer<Payload> {
    @Override
    public Payload deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        Payload payload;

        switch (type) {
            case 5: {
                payload = new Gson().fromJson(json, TaggedDataPayload.class);
                break;
            }
            case 6: {
                payload = new Gson().fromJson(json, TransactionPayload.class);
                break;
            }
            default: throw new JsonParseException("unknown type: " + type);
        }

        return payload;
    }
}

