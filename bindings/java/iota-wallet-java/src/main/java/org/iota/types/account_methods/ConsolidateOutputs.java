package org.iota.types.account_methods;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

public class ConsolidateOutputs implements AccountMethod {

    private boolean force;
    private Integer outputConsolidationThreshold;

    public ConsolidateOutputs withForce(boolean force) {
        this.force = force;
        return this;
    }

    public ConsolidateOutputs withOutputConsolidationThreshold(Integer outputConsolidationThreshold) {
        this.outputConsolidationThreshold = outputConsolidationThreshold;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("nftId", force);
        o.addProperty("outputConsolidationThreshold", outputConsolidationThreshold);

        return o;
    }

}