// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.ids.TokenId;

public class NativeTokensBalance extends AbstractObject {
    /// Token id
    private TokenId tokenId;
    /// Total amount
    private String total;
    /// Balance that can currently be spent
    private String available;
    /// Token foundry immutable metadata
    private String metadata;

    public TokenId getTokenId() {
        return tokenId;
    }

    public String getTotal() {
        return total;
    }

    public String getAvailable() {
        return available;
    }

    public String getMetadata() {
        return metadata;
    }

}
