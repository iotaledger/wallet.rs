// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

public class RequestFundsFromFaucet implements AccountMethod {

    private String faucetUrl;
    private String address;

    public RequestFundsFromFaucet(String faucetUrl, String address) {
        this.faucetUrl = faucetUrl;
        this.address = address;
    }
}