package org.iota.types;

import com.google.gson.JsonObject;
import com.google.gson.JsonSyntaxException;

public class OutputOptions extends AbstractObject {
    public OutputOptions(JsonObject jsonObject) {
        super(jsonObject);
    }

    public OutputOptions(String jsonObject) throws JsonSyntaxException {
        super(jsonObject);
    }
}
