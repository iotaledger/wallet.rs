// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.token_scheme;
public class SimpleTokenScheme extends TokenScheme {
    private int type = 0;
    private String mintedTokens;
    private String meltedTokens;
    private String maximumSupply;

    public int getType() {
        return type;
    }

    public String getMintedTokens() {
        return mintedTokens;
    }

    public SimpleTokenScheme withMintedTokens(String mintedTokens) {
        this.mintedTokens = mintedTokens;
        return this;
    }

    public String getMeltedTokens() {
        return meltedTokens;
    }

    public SimpleTokenScheme withMeltedTokens(String meltedTokens) {
        this.meltedTokens = meltedTokens;
        return this;
    }

    public String getMaximumSupply() {
        return maximumSupply;
    }

    public SimpleTokenScheme withMaximumSupply(String maximumSupply) {
        this.maximumSupply = maximumSupply;
        return this;
    }
}