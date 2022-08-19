package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ids.TransactionId;

public class GetIncomingTransactionData implements AccountMethod {

    private TransactionId transactionId;

    public GetIncomingTransactionData withTransactionId(TransactionId transactionId) {
        this.transactionId = transactionId;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("transactionId", transactionId.toString());

        return o;
    }
}