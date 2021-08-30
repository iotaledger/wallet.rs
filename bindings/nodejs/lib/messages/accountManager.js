// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const mh = require("./messageHandler.js")
const acc = require('./account.js');
let { MessageHandler } = mh
let { AccountForMessages } = acc;

class AccountManagerForMessages {
  constructor(options) {
    this.messageHandler = new MessageHandler(options);
  }
  async getAccount(accountId) {
    return this.messageHandler.sendMessage({
      cmd: "GetAccount",
      payload: accountId,
    }).then((acc) => new AccountForMessages(JSON.parse(acc).payload, this.messageHandler));
  }

  async getAccounts() {
    return this.messageHandler.sendMessage({
      cmd: "GetAccounts",
    });
  }

  async createAccount(account) {
    return this.messageHandler.sendMessage({
      cmd: "CreateAccount",
      payload: account
    }).then((acc) => new AccountForMessages(JSON.parse(acc).payload, this.messageHandler));
  }

  async setStrongholdPassword(password) {
    return this.messageHandler.sendMessage({
      cmd: "SetStrongholdPassword",
      payload: password,
    });
  }

  async storeMnemonic(mnemonic) {
    return this.messageHandler.sendMessage({
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
      cmd: "Backup",
      payload: {
          destination,
          password,
      },
    });
  }

  async importAccounts(backupPath, password) {
    return this.messageHandler.sendMessage({
      cmd: "RestoreBackup",
      payload: {
          destination,
          password,
      },
    });
  }
};

module.exports.AccountManagerForMessages = AccountManagerForMessages;
