// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.signature;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.features.*;

import java.lang.reflect.Type;
@JsonAdapter(SignatureAdapter.class)
public abstract class Signature {}

class SignatureAdapter implements JsonDeserializer<Signature>, JsonSerializer<Signature> {

    @Override
    public Signature deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        Signature signature;

        switch (type) {
            case 0: {
                signature = new Gson().fromJson(json, Ed25519Signature.class);
                break;
            }

            default: throw new JsonParseException("unknown type: " + type);
        }

        return signature;
    }

    public JsonElement serialize(Signature src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof Ed25519Signature) {
            return new Gson().toJsonTree(src, Ed25519Signature.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }

}