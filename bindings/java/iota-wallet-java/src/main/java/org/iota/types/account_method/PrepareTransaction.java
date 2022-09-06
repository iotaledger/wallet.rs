package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.OutputOptions;

public class PrepareTransaction implements AccountMethod {

    private OutputOptions options;
    private TransactionOptions transactionOptions;

    public PrepareTransaction withOptions(OutputOptions options) {
        this.options = options;
        return this;
    }

    public PrepareTransaction withTransactionOptions(TransactionOptions transactionOptions) {
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