package org.iota.types.payload;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(PayloadAdapter.class)
public abstract class Payload {}

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
            default: throw new JsonParseException("unknown type: " + type);
        }

        return payload;
    }
}

