// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.JsonObject;

public class WalletEvent extends AbstractJsonObject {

    public WalletEvent(JsonObject jsonObject) {
        super(jsonObject);
    }

    public WalletEvent(String jsonObject) {
        super(jsonObject);
    }

}