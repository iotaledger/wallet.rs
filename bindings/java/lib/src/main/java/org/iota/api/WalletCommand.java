package org.iota.api;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

public class WalletCommand {

    private String methodName;
    private JsonElement methodParams;

    public WalletCommand(String methodName) {
        this.methodName = methodName;
    }

    public String getMethodName() {
        return methodName;
    }

    public WalletCommand(String methodName, JsonElement methodParams) {
        this.methodName = methodName;
        this.methodParams = methodParams;
    }

    @Override
    public String toString() {
        JsonObject message = new JsonObject();
        message.addProperty("cmd", methodName);
        message.add("payload", methodParams);

        return message.toString();
    }
}