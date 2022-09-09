package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ids.NftId;

public class BurnNft implements AccountMethod {

    private NftId nftId;
    private TransactionOptions transactionOptions;

    public BurnNft withNftId(NftId nftId) {
        this.nftId = nftId;
        return this;
    }

    public BurnNft withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("nftId", nftId.toString());
        o.add("transactionOptions", transactionOptions.toJson());

        return o;
    }
}