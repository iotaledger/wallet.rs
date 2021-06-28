// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
const utils = require('./utils.js');
const ac = require('./account.js');
console.log(utils);
let { sendMessage, messageHandlerNew } = addon;
let { Account } = ac;

const sendMessageAsync = utils.promisify(sendMessage);
const _id = "1";
class MessageHandler {
  constructor(options) {
    console.log("MessageHandler constructor called.");
    this.messageHandler = messageHandlerNew(JSON.stringify(options));
  }

  async sendMessage(message) {
    console.log("sendMessage called.");
    console.log(message);
    return sendMessageAsync(JSON.stringify(message), this.messageHandler);
  }

  async getAccount(accountId) {
    return this.sendMessage({
      id: _id,
      cmd: "GetAccount",
      payload: accountId,
    }).then((acc) => new Account(JSON.parse(acc).payload, this));
  }

  async getAccounts() {
    return this.sendMessage({
      id: _id,
      cmd: "GetAccounts",
    });
  }

  async createAccount(account) {
    return this.sendMessage({
      id: _id,
      cmd: "CreateAccount",
      payload: account
    }).then((acc) => new Account(JSON.parse(acc).payload, this));
  }

  async setStrongholdPassword(password) {
    return this.sendMessage({
      id: _id,
      cmd: "SetStrongholdPassword",
      payload: password,
    });
  }

  async storeMnemonic(mnemonic) {
    return this.sendMessage({
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

};

module.exports.MessageHandler = MessageHandler;
