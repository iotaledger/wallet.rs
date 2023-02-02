// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import java.util.Map;

public class NodeAuth extends AbstractObject {
    /// JWT.
    private String jwt;
    /// Username and password.
    private Map.Entry<String, String> basic_auth_name_pwd;

    public String getJwt() {
        return jwt;
    }

    public NodeAuth withJwt(String jwt) {
        this.jwt = jwt;
        return this;
    }

    public Map.Entry<String, String> getBasic_auth_name_pwd() {
        return basic_auth_name_pwd;
    }

    public NodeAuth withBasic_auth_name_pwd(Map.Entry<String, String> basic_auth_name_pwd) {
        this.basic_auth_name_pwd = basic_auth_name_pwd;
        return this;
    }
}