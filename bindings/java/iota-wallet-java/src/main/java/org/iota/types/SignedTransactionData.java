// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonObject;

public class SignedTransactionData extends AbstractObject {

    public SignedTransactionData(JsonObject jsonObject) {
        super(jsonObject);
    }

    public SignedTransactionData(String jsonObject) {
        super(jsonObject);
    }

}