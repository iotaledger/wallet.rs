// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');

class SyncedAccount {
    constructor(id) {
      this.id = id;
    }

  };

module.exports.SyncedAccount = SyncedAccount;
