// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.outputs;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;
import org.iota.types.inputs.Input;
import org.iota.types.inputs.TreasuryInput;
import org.iota.types.inputs.UtxoInput;

import java.lang.reflect.Type;
@JsonAdapter(OutputAdapter.class)
public abstract class Output extends AbstractObject {}

class OutputAdapter implements JsonDeserializer<Output>, JsonSerializer<Output>{
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

    public JsonElement serialize(Output src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof AliasOutput) {
            return new Gson().toJsonTree(src, AliasOutput.class);
        } else if (src instanceof BasicOutput) {
            return new Gson().toJsonTree(src, BasicOutput.class);
        } else if (src instanceof FoundryOutput) {
            return new Gson().toJsonTree(src, FoundryOutput.class);
        } else if (src instanceof NftOutput) {
            return new Gson().toJsonTree(src, NftOutput.class);
        } else if (src instanceof TreasuryOutput) {
            return new Gson().toJsonTree(src, TreasuryOutput.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }
}