// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.transaction_essence;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;
import org.iota.types.AbstractObject;
import org.iota.types.unlock_conditions.*;

import java.lang.reflect.Type;
@JsonAdapter(TransactionEssenceAdapter.class)
public abstract class TransactionEssence extends AbstractObject {}

class TransactionEssenceAdapter implements JsonDeserializer<TransactionEssence>, JsonSerializer<TransactionEssence> {
    @Override
    public TransactionEssence deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        int type = json.getAsJsonObject().get("type").getAsInt();

        TransactionEssence transactionEssence;

        switch (type) {
            case 1: {
                transactionEssence = new Gson().fromJson(json, RegularTransactionEssence.class);
                break;
            }
            default: throw new JsonParseException("unknown type: " + type);
        }

        return transactionEssence;
    }
    @Override
    public JsonElement serialize(TransactionEssence src, Type typeOfSrc, JsonSerializationContext context) {
        if (src instanceof RegularTransactionEssence) {
            return new Gson().toJsonTree(src, RegularTransactionEssence.class);
        } else throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());
    }
}
