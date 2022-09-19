package org.iota.types.transaction_essence;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import java.lang.reflect.Type;

@JsonAdapter(TransactionEssenceAdapter.class)
public abstract class TransactionEssence {}

class TransactionEssenceAdapter implements JsonDeserializer<TransactionEssence> {
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
}
