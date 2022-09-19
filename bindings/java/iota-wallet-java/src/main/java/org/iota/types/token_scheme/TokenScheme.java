package org.iota.types.token_scheme;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(TokenSchemeAdapter.class)
public abstract class TokenScheme  {}

class TokenSchemeAdapter implements JsonDeserializer<TokenScheme> {
    @Override
    public TokenScheme deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        TokenScheme tokenScheme;

        switch (type) {
            case 0: {
                tokenScheme = new Gson().fromJson(json, SimpleTokenScheme.class);
                break;
            }
            default: throw new JsonParseException("unknown type: " + type);
        }

        return tokenScheme;
    }
}

