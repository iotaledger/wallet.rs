package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.JsonUtils;
import org.iota.types.NftOptions;

public class MintNfts implements AccountMethod {

    private NftOptions[] nftOptions;
    private TransactionOptions options;

    public MintNfts withNftOptions(NftOptions[] nftOptions) {
        this.nftOptions = nftOptions;
        return this;
    }

    public MintNfts withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("nftOptions", JsonUtils.toJson(nftOptions));
        o.add("options", options.toJson());

        return o;
    }
}