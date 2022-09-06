package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class OutputsToClaim extends AbstractObject {
    public OutputsToClaim(JsonObject jsonObject) {
        super(jsonObject);
    }

    public OutputsToClaim(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
