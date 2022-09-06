package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.OutputOptions;

public class PrepareOutput implements AccountMethod {

    private OutputOptions options;
    private TransactionOptions transactionOptions;

    public PrepareOutput withOptions(OutputOptions options) {
        this.options = options;
        return this;
    }

    public PrepareOutput withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("options", options.toJson());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}