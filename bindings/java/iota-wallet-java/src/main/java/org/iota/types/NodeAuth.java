// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonObject;

public class NodeAuth extends AbstractObject {

    public NodeAuth(JsonObject jsonObject) {
        super(jsonObject);
    }

    public NodeAuth(String jsonObject) {
        super(jsonObject);
    }

}