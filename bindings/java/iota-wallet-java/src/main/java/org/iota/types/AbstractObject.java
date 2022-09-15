package org.iota.types;

import com.google.gson.Gson;
import com.google.gson.JsonElement;

import java.util.Objects;

public abstract class AbstractObject {

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        JsonElement e = new Gson().toJsonTree(this);
        JsonElement other = new Gson().toJsonTree(o);
        return Objects.equals(e, other);
    }

    @Override
    public int hashCode() {
        return new Gson().toJsonTree(this).hashCode();
    }

    @Override
    public String toString() {
        return new Gson().toJsonTree(this).toString();
    }
}
