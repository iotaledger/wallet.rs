// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.exceptions;

public class WalletException extends Exception {

    private String methodName;

    public WalletException(String methodName, String message) {
        super(message);
        this.methodName = methodName;
    }

    public String getMethodName() {
        return methodName;
    }

}