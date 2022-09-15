package org.iota.types;

import org.iota.types.ids.TokenId;

public class NativeTokensBalance extends AbstractObject {
    /// Token id
    private TokenId tokenId;
    /// Total amount
    private String total;
    /// Balance that can currently be spent
    private String available;

    public TokenId getTokenId() {
        return tokenId;
    }

    public String getTotal() {
        return total;
    }

    public String getAvailable() {
        return available;
    }
}
