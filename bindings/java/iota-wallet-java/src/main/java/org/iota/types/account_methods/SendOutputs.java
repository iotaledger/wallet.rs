package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.JsonUtils;
import org.iota.types.Output;

public class SendOutputs implements AccountMethod {

    private Output[] outputs;
    private TransactionOptions options;

    public SendOutputs withOutputs(Output[] outputs) {
        this.outputs = outputs;
        return this;
    }

    public SendOutputs withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("addressesWithMicroAmount", JsonUtils.toJson(outputs));
        o.add("options", options.toJson());

        return o;
    }
}