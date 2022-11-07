// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transactionprogress;

import java.lang.reflect.Type;
import java.util.Set;
import java.util.Map.Entry;

import org.iota.types.AbstractObject;
import org.iota.types.events.wallet.TransactionProgress;

import com.google.gson.Gson;
import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.annotations.JsonAdapter;

@JsonAdapter(TransactionProgressEventAdapter.class)
public abstract class TransactionProgressEvent extends TransactionProgress {

    protected TransactionProgressEventType transactionType;

    public TransactionProgressEventType getTransactionType() {
        return transactionType;
    }
}

class TransactionProgressEventAdapter implements JsonDeserializer<TransactionProgressEvent> {
    @Override
    public TransactionProgressEvent deserialize(final JsonElement json, final Type typeOfT,
            final JsonDeserializationContext context)
            throws JsonParseException {
        JsonElement jsonType = json.getAsJsonObject().get("TransactionProgress");

        String type;
        // Ennum with value
        if (jsonType.isJsonObject()) {
            type = jsonType.getAsJsonObject().entrySet().iterator().next().getKey();
        } else {
            // Enum without vaue
            type = jsonType.getAsString();
        }

        TransactionProgressEvent event;

        switch (type) {
            case "SelectingInputs": {
                event = new Gson().fromJson("{}", SelectingInputs.class);
                break;
            }
            case "GeneratingRemainderDepositAddress": { // AddressData
                event = new Gson().fromJson(jsonType, GeneratingRemainderDepositAddress.class);
                break;
            }
            case "PreparedTransaction": { // PreparedTransactionDataDto
                event = new Gson().fromJson(jsonType, PreparedTransaction.class);
                break;
            }
            case "Broadcasting": {
                event = new Gson().fromJson("{}", Broadcasting.class);
                break;
            }
            case "PerformingPow": {
                event = new Gson().fromJson("{}", PerformingPow.class);
                break;
            }
            case "SigningTransaction": {
                event = new Gson().fromJson("{}", SigningTransaction.class);
                break;
            }
            case "PreparedTransactionEssenceHash": { // String hash
                event = new Gson().fromJson(jsonType, PreparedTransactionEssenceHash.class);
                break;
            }
            default:
                throw new JsonParseException("unknown event type: " + type);
        }

        event.transactionType = TransactionProgressEventType.valueOf(type);
        return event;
    }
}
