package org.iota.types.outputs;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(OutputAdapter.class)
public abstract class Output {}

class OutputAdapter implements JsonDeserializer<Output> {
    @Override
    public Output deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        Output output;

        switch (type) {
            case 2: {
                output = new Gson().fromJson(json, TreasuryOutput.class);
                break;
            }
            case 3: {
                output = new Gson().fromJson(json, BasicOutput.class);
                break;
            }
            case 4: {
                output = new Gson().fromJson(json, AliasOutput.class);
                break;
            }
            case 5: {
                output = new Gson().fromJson(json, FoundryOutput.class);
                break;
            }
            case 6: {
                output = new Gson().fromJson(json, NftOutput.class);
                break;
            }
            default: throw new JsonParseException("unknown type: " + type);
        }

        return output;
    }
}