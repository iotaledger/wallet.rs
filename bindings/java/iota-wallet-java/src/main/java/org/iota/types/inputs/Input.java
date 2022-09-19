package org.iota.types.inputs;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(InputAdapter.class)
public abstract class Input {}

class InputAdapter implements JsonDeserializer<Input> {
    @Override
    public Input deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        Input input;

        switch (type) {
            case 0: {
                input = new Gson().fromJson(json, UtxoInput.class);
                break;
            }
            case 1: {
                input = new Gson().fromJson(json, TreasuryInput.class);
                break;
            }
            default: throw new JsonParseException("unknown type: " + type);
        }

        return input;
    }
}

