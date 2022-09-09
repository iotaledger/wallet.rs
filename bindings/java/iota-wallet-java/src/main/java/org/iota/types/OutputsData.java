// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonObject;

public class OutputsData extends AbstractObject {

    public OutputsData(JsonObject jsonObject) {
        super(jsonObject);
    }

    public OutputsData(String jsonObject) {
        super(jsonObject);
    }

}