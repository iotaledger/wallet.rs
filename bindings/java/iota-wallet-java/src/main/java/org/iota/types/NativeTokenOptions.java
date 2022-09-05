package org.iota.types;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.account_method.AccountMethod;

public class NativeTokenOptions implements ReturnJson {

    private String accountAddress;
    private String circulatingSupply;
    private String maximumSupply;
    private String foundryMetadata;

    public NativeTokenOptions withAccountAddress(String accountAddress) {
        this.accountAddress = accountAddress;
        return this;
    }

    public NativeTokenOptions withCirculating_supply(String circulatingSupply) {
        this.circulatingSupply = circulatingSupply;
        return this;
    }

    public NativeTokenOptions withMaximumSupply(String maximumSupply) {
        this.maximumSupply = maximumSupply;
        return this;
    }

    public NativeTokenOptions withFoundryMetadata(String foundryMetadata) {
        this.foundryMetadata = foundryMetadata;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("accountAddress", accountAddress);
        o.addProperty("circulatingSupply", circulatingSupply);
        o.addProperty("maximumSupply", maximumSupply);
        o.addProperty("foundryMetadata", foundryMetadata);

        return o;
    }

}