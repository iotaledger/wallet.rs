package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.ReturnJson;
import org.iota.types.ids.OutputId;

public class GetOutput implements AccountMethod {

    private OutputId outputId;

    public GetOutput withOutputId(OutputId outputId) {
        this.outputId = outputId;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("outputId", outputId.toString());

        return o;
    }
}