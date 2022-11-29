// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transaction;

import java.lang.reflect.Type;
import java.util.Map.Entry;

import org.iota.api.CustomGson;
import org.iota.types.PreparedTransactionData;
import org.iota.types.events.wallet.WalletEvent;
import org.iota.types.events.wallet.WalletEventType;

import com.google.gson.Gson;
import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParseException;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.JsonAdapter;

@JsonAdapter(TransactionProgressEventAdapter.class)
public abstract class TransactionProgressEvent extends WalletEvent {

    protected transient TransactionProgressEventType transactionType;

    public TransactionProgressEvent(TransactionProgressEventType transactionType) {
        super(WalletEventType.TransactionProgress);
        this.transactionType = transactionType;
    }

    public TransactionProgressEventType getTransactionType() {
        return transactionType;
    }
}

class TransactionProgressEventAdapter
        implements JsonDeserializer<TransactionProgressEvent>, JsonSerializer<TransactionProgressEvent> {
    @Override
    public TransactionProgressEvent deserialize(final JsonElement json, final Type typeOfT,
            final JsonDeserializationContext context)
            throws JsonParseException {
        JsonElement jsonType = json.getAsJsonObject().get("TransactionProgress");

        String type;
        JsonElement value;
        // Ennum with value
        if (jsonType.isJsonObject()) {
            Entry<String, JsonElement> entry = jsonType.getAsJsonObject().entrySet().iterator().next();
            type = entry.getKey();
            value = entry.getValue();
        } else {
            // Enum without vaue
            type = jsonType.getAsString();
            value = new JsonObject();
        }

        TransactionProgressEvent event;

        switch (type) {
            case "SelectingInputs": {
                event = CustomGson.get().fromJson(value, SelectingInputs.class);
                break;
            }
            case "GeneratingRemainderDepositAddress": {
                event = CustomGson.get().fromJson(value, GeneratingRemainderDepositAddress.class);
                break;
            }
            case "PreparedTransaction": {
                event = CustomGson.get().fromJson(value, PreparedTransaction.class);
                break;
            }
            case "Broadcasting": {
                event = CustomGson.get().fromJson(value, Broadcasting.class);
                break;
            }
            case "PerformingPow": {
                event = CustomGson.get().fromJson(value, PerformingPow.class);
                break;
            }
            case "SigningTransaction": {
                event = CustomGson.get().fromJson(value, SigningTransaction.class);
                break;
            }
            case "PreparedTransactionEssenceHash": {
                event = new PreparedTransactionEssenceHash(jsonType.getAsJsonObject().get(type).getAsString());
                break;
            }
            default:
                throw new JsonParseException("unknown event type: " + type);
        }

        event.transactionType = TransactionProgressEventType.valueOf(type);
        return event;
    }

    @Override
    public JsonElement serialize(TransactionProgressEvent src, Type typeOfSrc, JsonSerializationContext context) {
        JsonObject element = new JsonObject();
        JsonElement value;
        if (src instanceof SelectingInputs) {
            value = new Gson().toJsonTree(src, SelectingInputs.class);
        } else if (src instanceof GeneratingRemainderDepositAddress) {
            value = new Gson().toJsonTree(src, GeneratingRemainderDepositAddress.class);
        } else if (src instanceof PreparedTransaction) {
            // Directy serialise the tx
            value = new Gson().toJsonTree(((PreparedTransaction) src).getPreparedTransaction(),
                    PreparedTransactionData.class);
        } else if (src instanceof Broadcasting) {
            value = new Gson().toJsonTree(src, Broadcasting.class);
        } else if (src instanceof PerformingPow) {
            value = new Gson().toJsonTree(src, PerformingPow.class);
        } else if (src instanceof SigningTransaction) {
            value = new Gson().toJsonTree(src, SigningTransaction.class);
        } else if (src instanceof PreparedTransactionEssenceHash) {
            value = new JsonPrimitive(((PreparedTransactionEssenceHash) src).getHash());
        } else
            throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());

        if (!value.isJsonObject() || value.getAsJsonObject().size() != 0) {
            element.add(src.transactionType.toString(), value);
            return element;
        } else {
            return new JsonPrimitive(src.transactionType.toString());
        }
    }
}
