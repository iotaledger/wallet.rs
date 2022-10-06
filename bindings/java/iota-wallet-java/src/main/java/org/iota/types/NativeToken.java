// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.TokenId;

public class NativeToken extends AbstractObject {

    // Identifier of the native token.
    private TokenId id;
    // Identifier of the native token.
    private String amount;

    public NativeToken(TokenId id, String amount) {
        this.id = id;
        this.amount = amount;
    }

    public TokenId getId() {
        return id;
    }

    public String getAmount() {
        return amount;
    }
}