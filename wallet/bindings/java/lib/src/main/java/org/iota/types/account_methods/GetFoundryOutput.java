// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.ids.TokenId;

/// Get the output that minted a native token by its TokenId
public class GetFoundryOutput implements AccountMethod {

    private TokenId tokenId;

    public GetFoundryOutput withTokenId(TokenId tokenId) {
        this.tokenId = tokenId;
        return this;
    }
}