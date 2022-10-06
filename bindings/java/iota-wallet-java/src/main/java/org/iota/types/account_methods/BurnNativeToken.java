// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.TransactionOptions;
import org.iota.types.ids.TokenId;

/// Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
/// the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
/// recommended to use melting, if the foundry output is available.
public class BurnNativeToken implements AccountMethod {

    private TokenId tokenId;
    /// To be burned amount
    private String burnAmount;
    private TransactionOptions transactionOptions;

    public BurnNativeToken withTokenId(TokenId tokenId) {
        this.tokenId = tokenId;
        return this;
    }

    public BurnNativeToken withBurnAmount(String burnAmount) {
        this.burnAmount = burnAmount;
        return this;
    }

    public BurnNativeToken withTransactionOptions(TransactionOptions transactionOptions) {
        this.transactionOptions = transactionOptions;
        return this;
    }
}