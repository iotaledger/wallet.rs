package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ids.OutputId;
import org.iota.types.ids.TokenId;

public class GetFoundryOutput implements AccountMethod {

    private TokenId tokenId;

    public GetFoundryOutput withTokenId(TokenId tokenId) {
        this.tokenId = tokenId;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("tokenId", tokenId.toString());

        return o;
    }
}