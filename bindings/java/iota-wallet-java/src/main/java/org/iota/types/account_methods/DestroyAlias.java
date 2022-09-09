package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ids.AliasId;

public class DestroyAlias implements AccountMethod {

    private AliasId aliasId;
    private TransactionOptions transactionOptions;

    public DestroyAlias withAliasId(AliasId aliasId) {
        this.aliasId = aliasId;
        return this;
    }

    public DestroyAlias withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("aliasId", aliasId.toString());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}