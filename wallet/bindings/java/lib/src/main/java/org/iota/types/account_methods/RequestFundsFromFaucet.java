// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types.account_methods;

public class RequestFundsFromFaucet implements AccountMethod {

    private String url;
    private String address;

    public RequestFundsFromFaucet(String url, String address) {
        this.url = url;
        this.address = address;
    }
}