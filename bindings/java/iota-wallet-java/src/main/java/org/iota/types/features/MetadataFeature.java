package org.iota.types.features;

public class MetadataFeature extends Feature {

    private int type = 2;
    private String data;

    public int getType() {
        return type;
    }

    public String getData() {
        return data;
    }

    public MetadataFeature withData(String data) {
        this.data = data;
        return this;
    }
}