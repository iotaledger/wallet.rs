package org.iota.types.token_scheme;

import com.google.gson.*;
import org.iota.types.unlock_conditions.*;

import java.lang.reflect.Type;

public abstract class TokenScheme implements JsonDeserializer<TokenScheme> {

    @Override
    public TokenScheme deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        JsonObject jsonObject = json.getAsJsonObject();

        int type = jsonObject.get("type").getAsInt();

        TokenScheme tokenScheme;

        switch (type) {
            case 0: {
                tokenScheme = new Gson().fromJson(json, TokenScheme.class);
                break;
            }

            default: throw new JsonParseException("unknown type: " + type);
        }

        return tokenScheme;
    }

}

