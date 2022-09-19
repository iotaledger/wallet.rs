package org.iota.types.signature;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

public abstract class Signature {}

@JsonAdapter(SignatureAdapter.class)
class SignatureAdapter implements JsonDeserializer<Signature> {

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

}