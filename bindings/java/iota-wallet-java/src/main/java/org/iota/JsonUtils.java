package org.iota;

import com.google.gson.JsonArray;
import org.iota.types.AbstractJsonObject;
import org.iota.types.ids.AbstractId;

public class JsonUtils<T> {


    public static JsonArray toJson(String[] array) {
        if (array != null) {
            JsonArray a = new JsonArray();
            for (String s : array) {
                a.add(s);
            }
            return a;
        } else {
            return null;
        }
    }

    public static JsonArray toJson(byte[] array) {
        if (array != null) {
            JsonArray a = new JsonArray();
            for (byte b : array) {
                a.add(b & 0xFF);
            }
            return a;
        } else {
            return null;
        }
    }

}
