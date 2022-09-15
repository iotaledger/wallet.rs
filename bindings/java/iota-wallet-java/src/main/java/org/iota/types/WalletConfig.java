// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.api.GsonSingleton;
import org.iota.types.secret.SecretManager;

public class WalletConfig {

    private String storagePath;
    private String clientOptions;
    private Integer coinType;
    private String secretManager;

    public WalletConfig withStoragePath(String storagePath) {
        this.storagePath = storagePath;
        return this;
    }

    public WalletConfig withClientOptions(ClientConfig clientOptions) {
        this.clientOptions = GsonSingleton.getInstance().toJsonTree(clientOptions).toString();
        return this;
    }

    public WalletConfig withCoinType(Integer coinType) {
        this.coinType = coinType;
        return this;
    }

    public WalletConfig withSecretManager(SecretManager secretManager) {
        this.secretManager = GsonSingleton.getInstance().toJsonTree(secretManager).toString();
        return this;
    }

}