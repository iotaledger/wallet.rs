// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.TokenId;

public class MintTokenTransaction extends AbstractObject {
    private TokenId tokenId;
    private Transaction transaction;

    public TokenId getTokenId() {
        return tokenId;
    }

    public Transaction getTransaction() {
        return transaction;
    }
}