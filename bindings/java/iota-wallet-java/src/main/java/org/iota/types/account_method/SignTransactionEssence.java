package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.AddressNativeTokens;
import org.iota.types.JsonUtils;
import org.iota.types.PreparedTransactionData;

public class SignTransactionEssence implements AccountMethod {

    private PreparedTransactionData preparedTransactionData;

    public SignTransactionEssence withPreparedTransactionData(PreparedTransactionData preparedTransactionData) {
        this.preparedTransactionData = preparedTransactionData;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("options", preparedTransactionData.toJson());

        return o;
    }
}