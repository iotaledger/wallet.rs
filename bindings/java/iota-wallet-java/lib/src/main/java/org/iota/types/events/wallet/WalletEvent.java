// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.wallet;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

import org.iota.api.CustomGson;
import org.iota.types.AbstractObject;
import org.iota.types.events.transaction.TransactionProgressEvent;

import java.lang.reflect.Type;
import java.util.Map.Entry;

@JsonAdapter(WalletEventAdapter.class)
public abstract class WalletEvent extends AbstractObject {

    protected transient WalletEventType type;

    public WalletEvent(WalletEventType type) {
        this.type = type;
    }

    public WalletEventType getType() {
        return type;
    }
}

class WalletEventAdapter implements JsonDeserializer<WalletEvent>, JsonSerializer<WalletEvent> {
    @Override
    public WalletEvent deserialize(final JsonElement json, final Type typeOfT, final JsonDeserializationContext context)
            throws JsonParseException {

        String type;
        JsonElement value;

        // Ennum with value
        if (json.isJsonObject()) {
            Entry<String, JsonElement> entry = json.getAsJsonObject().entrySet().iterator().next();
            type = entry.getKey();
            value = entry.getValue();
        } else {
            // Enum without vaue
            type = json.getAsString();
            value = new JsonObject();
        }

        WalletEvent event;
        switch (type) {
            case "ConsolidationRequired": {
                event = CustomGson.get().fromJson(value, ConsolidationRequired.class);
                break;
            }
            case "LedgerAddressGeneration": {
                event = CustomGson.get().fromJson(value, LedgerAddressGeneration.class);
                break;
            }
            case "NewOutput": { // NewOutputEvent
                event = CustomGson.get().fromJson(value, NewOutput.class);
                break;
            }
            case "SpentOutput": {
                event = CustomGson.get().fromJson(value, SpentOutput.class);
                break;
            }
            case "TransactionInclusion": {
                event = CustomGson.get().fromJson(value, TransactionInclusion.class);
                break;
            }
            case "TransactionProgress": {
                // Needs the whole object to deterine enum type
                event = CustomGson.get().fromJson(json, TransactionProgressEvent.class);
                break;
            }
            default:
                throw new JsonParseException("unknown wallet event type: " + type);
        }

        event.type = WalletEventType.valueOf(type);
        return event;
    }

    @Override
    public JsonElement serialize(WalletEvent src, Type typeOfSrc, JsonSerializationContext context) {
        JsonObject element = new JsonObject();
        JsonElement value;
        if (src instanceof ConsolidationRequired) {
            value = new Gson().toJsonTree(src, ConsolidationRequired.class);
        } else if (src instanceof LedgerAddressGeneration) {
            value = new Gson().toJsonTree(src, LedgerAddressGeneration.class);
        } else if (src instanceof NewOutput) {
            value = new Gson().toJsonTree(src, NewOutput.class);
        } else if (src instanceof SpentOutput) {
            value = new Gson().toJsonTree(src, SpentOutput.class);
        } else if (src instanceof TransactionInclusion) {
            value = new Gson().toJsonTree(src, TransactionInclusion.class);
        } else if (src instanceof TransactionProgressEvent) {
            value = new Gson().toJsonTree(src, TransactionProgressEvent.class);
        } else
            throw new JsonParseException("unknown class: " + src.getClass().getSimpleName());

        if (!value.isJsonObject() || value.getAsJsonObject().size() != 0) {
            element.add(src.type.toString(), value);
            return element;
        } else {
            return new JsonPrimitive(src.type.toString());
        }
    }
}
