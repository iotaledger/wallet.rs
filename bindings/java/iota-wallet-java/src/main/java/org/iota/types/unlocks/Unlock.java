package org.iota.types.unlocks;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.inputs.Input;
import org.iota.types.inputs.TreasuryInput;
import org.iota.types.inputs.UtxoInput;

import java.lang.reflect.Type;

@JsonAdapter(UnlockAdapter.class)
public abstract class Unlock {}

class UnlockAdapter implements JsonDeserializer<Unlock>, JsonSerializer<Unlock> {

    @Override
    public Unlock deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        Unlock unlock;

        switch (type) {
            case 0: {
                unlock = new Gson().fromJson(json, SignatureUnlock.class);
                break;
            }
            case 1: {
                unlock = new Gson().fromJson(json, ReferenceUnlock.class);
                break;
            }
            case 2: {
                unlock = new Gson().fromJson(json, AliasUnlock.class);
                break;
            }
            case 3: {
                unlock = new Gson().fromJson(json, NftUnlock.class);
                break;
            }

            default: throw new JsonParseException("unknown type: " + type);
        }

        return unlock;
    }

    @Override
    public JsonElement serialize(Unlock src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof SignatureUnlock) {
            return new Gson().toJsonTree(src, SignatureUnlock.class);
        } else if (src instanceof ReferenceUnlock) {
            return new Gson().toJsonTree(src, ReferenceUnlock.class);
        } else if (src instanceof AliasUnlock) {
            return new Gson().toJsonTree(src, AliasUnlock.class);
        } else if (src instanceof NftUnlock) {
            return new Gson().toJsonTree(src, NftUnlock.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }

}