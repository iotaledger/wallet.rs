// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

import org.iota.types.NativeTokenOptions;
import org.iota.types.TransactionOptions;

/// Mint native token.
public class MintNativeToken implements AccountMethod {

    private NativeTokenOptions nativeTokenOptions;
    private TransactionOptions options;

    public MintNativeToken withNativeTokenOptions(NativeTokenOptions nativeTokenOptions) {
        this.nativeTokenOptions = nativeTokenOptions;
        return this;
    }

    public MintNativeToken withOptions(TransactionOptions options) {
        this.options = options;
        return this;
    }
}