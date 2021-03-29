// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.wallet.local;

public class WalletException extends RuntimeException {

    public WalletException() {
        super();
    }

    public WalletException(String errorMessage) {
        super(errorMessage);
    }
}
