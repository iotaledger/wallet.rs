package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.NativeToken;

public class MeltNativeToken implements AccountMethod {

    private NativeToken nativeToken;
    private TransactionOptions transactionOptions;

    public MeltNativeToken withNativeToken(NativeToken nativeToken) {
        this.nativeToken = nativeToken;
        return this;
    }

    public MeltNativeToken withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("nativeToken", nativeToken.toJson());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}