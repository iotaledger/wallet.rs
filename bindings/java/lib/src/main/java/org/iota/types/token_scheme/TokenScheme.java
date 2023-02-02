// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.token_scheme;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.transaction_essence.RegularTransactionEssence;
import org.iota.types.transaction_essence.TransactionEssence;

import java.lang.reflect.Type;
@JsonAdapter(TokenSchemeAdapter.class)
public abstract class TokenScheme {}

class TokenSchemeAdapter implements JsonDeserializer<TokenScheme>, JsonSerializer<TokenScheme> {
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

    public JsonElement serialize(TokenScheme src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof SimpleTokenScheme) {
            return new Gson().toJsonTree(src, SimpleTokenScheme.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }
}

