// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import java.util.Map;

public class NodeAuth extends NewAbstractObject {
    /// JWT.
    private String jwt;
    /// Username and password.
    private Map.Entry<String, String> basic_auth_name_pwd;

}