package org.iota.types.inputs;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;

import java.lang.reflect.Type;

@JsonAdapter(InputAdapter.class)
public abstract class Input extends AbstractObject {}

class InputAdapter implements JsonDeserializer<Input>, JsonSerializer<Input> {
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

    @Override
    public JsonElement serialize(Input src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof UtxoInput) {
            return new Gson().toJsonTree(src, UtxoInput.class);
        } else if (src instanceof TreasuryInput) {
            return new Gson().toJsonTree(src, TreasuryInput.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }

}

