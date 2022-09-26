// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.payload;
public class TaggedDataPayload extends Payload {
    private int type;
    private String tag;
    private String data;

    public TaggedDataPayload(String tag, String data) {
        this.tag = tag;
        this.data = data;
    }

    public String getTag() {
        return tag;
    }

    public String getData() {
        return data;
    }
}