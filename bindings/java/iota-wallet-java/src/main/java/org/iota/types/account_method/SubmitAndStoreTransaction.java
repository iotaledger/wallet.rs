package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.PreparedTransactionData;
import org.iota.types.SignedTransactionData;

public class SubmitAndStoreTransaction implements AccountMethod {

    private SignedTransactionData signedTransactionData;

    public SubmitAndStoreTransaction withSignedTransactionData(SignedTransactionData signedTransactionData) {
        this.signedTransactionData = signedTransactionData;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("signedTransactionData", signedTransactionData.toJson());

        return o;
    }
}