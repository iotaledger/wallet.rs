// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import com.google.gson.Gson;
import org.iota.api.CustomGson;
import org.iota.types.secret.SecretManager;

public class WalletConfig extends AbstractObject {

    private String storagePath;
    private String clientOptions;
    private Integer coinType;
    private String secretManager;

    public WalletConfig withStoragePath(String storagePath) {
        this.storagePath = storagePath;
        return this;
    }

    public WalletConfig withClientOptions(ClientConfig clientOptions) {
        this.clientOptions = new Gson().toJsonTree(clientOptions).toString();
        return this;
    }

    public WalletConfig withCoinType(Integer coinType) {
        this.coinType = coinType;
        return this;
    }

    public WalletConfig withSecretManager(SecretManager secretManager) {
        this.secretManager = CustomGson.get().toJsonTree(secretManager).toString();
        return this;
    }

}