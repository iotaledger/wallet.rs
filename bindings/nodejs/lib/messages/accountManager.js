// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const mh = require("./messageHandler.js")
const acc = require('./account.js');
let { MessageHandler } = mh
let { Account } = acc;

const _id = "1";

class AccountManager {
  constructor(options) {
    this.messageHandler = new MessageHandler(options);
  }
  async getAccount(accountId) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "GetAccount",
      payload: accountId,
    }).then((acc) => new Account(JSON.parse(acc).payload, this.messageHandler));
  }

  async getAccounts() {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "GetAccounts",
    });
  }

  async createAccount(account) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "CreateAccount",
      payload: account
    }).then((acc) => new Account(JSON.parse(acc).payload, this.messageHandler));
  }

  async setStrongholdPassword(password) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "SetStrongholdPassword",
      payload: password,
    });
  }

  async storeMnemonic(mnemonic) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "StoreMnemonic",
      payload: {
        signerType: {
          type: 'Stronghold'
        },
        mnemonic
      },
    });
  }

  async backup(destination, password) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "Backup",
      payload: {
          destination,
          password,
      },
    });
  }

  async importAccounts(backupPath, password) {
    return this.messageHandler.sendMessage({
      id: _id,
      cmd: "RestoreBackup",
      payload: {
          destination,
          password,
      },
    });
  }
};

module.exports.AccountManager = AccountManager;
