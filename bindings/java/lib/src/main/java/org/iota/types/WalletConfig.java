// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.secret.SecretManager;

public class WalletConfig extends AbstractObject {

    private String storagePath;
    private ClientConfig clientOptions;
    private Integer coinType;
    private SecretManager secretManager;

    public WalletConfig withStoragePath(String storagePath) {
        this.storagePath = storagePath;
        return this;
    }

    public WalletConfig withClientOptions(ClientConfig clientOptions) {
        this.clientOptions = clientOptions;
        return this;
    }

    public WalletConfig withCoinType(CoinType coinType) {
        this.coinType = coinType.getCoinTypeValue();
        return this;
    }

    public WalletConfig withSecretManager(SecretManager secretManager) {
        this.secretManager = secretManager;
        return this;
    }

}