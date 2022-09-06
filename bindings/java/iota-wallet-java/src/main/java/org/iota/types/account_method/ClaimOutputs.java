package org.iota.types.account_method;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.JsonUtils;
import org.iota.types.OutputsToClaim;
import org.iota.types.ids.OutputId;

public class ClaimOutputs implements AccountMethod {

    private OutputId[] outputIdsToClaim;


    public ClaimOutputs withOutputsToClaim(OutputId[] outputIdsToClaim) {
        this.outputIdsToClaim = outputIdsToClaim;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("outputIdsToClaim", JsonUtils.toJson(outputIdsToClaim));

        return o;
    }
}