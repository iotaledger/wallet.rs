package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ids.FoundryId;

public class DestroyFoundry implements AccountMethod {

    private FoundryId foundryId;
    private TransactionOptions transactionOptions;

    public DestroyFoundry withAliasId(FoundryId foundryId) {
        this.foundryId = foundryId;
        return this;
    }

    public DestroyFoundry withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("foundryId", foundryId.toString());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}