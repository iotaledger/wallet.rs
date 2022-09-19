package org.iota.types.unlocks;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(UnlockAdapter.class)
public abstract class Unlock {}

class UnlockAdapter implements JsonDeserializer<Unlock> {

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

}