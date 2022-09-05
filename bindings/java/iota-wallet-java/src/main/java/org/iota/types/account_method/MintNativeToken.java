package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.NativeTokenOptions;

public class MintNativeToken implements AccountMethod {

    private NativeTokenOptions nativeTokenOptions;
    private TransactionOptions transactionOptions;


    public MintNativeToken withNativeTokenOptions(NativeTokenOptions nativeTokenOptions) {
        this.nativeTokenOptions = nativeTokenOptions;
        return this;
    }

    public MintNativeToken withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("nativeToken", nativeTokenOptions.toJson());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}