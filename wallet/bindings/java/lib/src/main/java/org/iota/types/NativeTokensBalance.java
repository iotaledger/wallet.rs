// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.TokenId;

public class NativeTokensBalance extends AbstractObject {
    /// Token id
    private TokenId tokenId;
    /// Token foundry immutable metadata
    private String metadata;
    /// Total amount
    private String total;
    /// Balance that can currently be spent
    private String available;

    public TokenId getTokenId() {
        return tokenId;
    }

    public String getMetadata() {
        return metadata;
    }

    public String getTotal() {
        return total;
    }

    public String getAvailable() {
        return available;
    }

}
