package org.iota.types;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;
import org.iota.types.account_method.AccountMethod;

public class NftOptions implements ReturnJson {

    private String address;
    private String immutableMetadata;
    private String metadata;

    public NftOptions withAddress(String address) {
        this.address = address;
        return this;
    }

    public NftOptions withImmutableMetadata(String immutableMetadata) {
        this.immutableMetadata = immutableMetadata;
        return this;
    }

    public NftOptions withMetadata(String metadata) {
        this.metadata = metadata;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("address", address);
        o.addProperty("immutableMetadata", immutableMetadata);
        o.addProperty("metadata", metadata);

        return o;
    }

}