// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.features;
public class TagFeature extends Feature {

    private int type = 3;
    private String tag;

    public int getType() {
        return type;
    }

    public String getTag() {
        return tag;
    }

    public TagFeature withTag(String tag) {
        this.tag = tag;
        return this;
    }
}