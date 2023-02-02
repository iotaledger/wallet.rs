// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.events.transaction;

import java.lang.reflect.Type;

import org.iota.api.CustomGson;
import org.iota.types.PreparedTransactionData;

import com.google.gson.*;
import com.google.gson.annotations.JsonAdapter;

@JsonAdapter(PreparedTransactionAdapter.class)
public class PreparedTransaction extends TransactionProgressEvent {

    PreparedTransactionData preparedTransaction;

    public PreparedTransaction(PreparedTransactionData preparedTransaction) {
        super(TransactionProgressEventType.PreparedTransaction);
        this.preparedTransaction = preparedTransaction;
    }

    public PreparedTransactionData getPreparedTransaction() {
        return preparedTransaction;
    }
}

class PreparedTransactionAdapter implements JsonDeserializer<PreparedTransaction> {
    @Override
    public PreparedTransaction deserialize(final JsonElement json, final Type typeOfT,
            final JsonDeserializationContext context)
            throws JsonParseException {
        return new PreparedTransaction(CustomGson.get().fromJson(json, PreparedTransactionData.class));
    }
}
