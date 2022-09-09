package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.OutputsToClaim;

public class TryClaimOutputs implements AccountMethod {

    private OutputsToClaim outputsToClaim;

    public TryClaimOutputs withOutputsToClaim(OutputsToClaim outputsToClaim) {
        this.outputsToClaim = outputsToClaim;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.add("outputsToClaim", outputsToClaim.toJson());

        return o;
    }
}