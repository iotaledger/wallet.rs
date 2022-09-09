// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonObject;

public class IncomingTransactionData extends AbstractObject {

    public IncomingTransactionData(JsonObject jsonObject) {
        super(jsonObject);
    }

    public IncomingTransactionData(String jsonObject) {
        super(jsonObject);
    }

}