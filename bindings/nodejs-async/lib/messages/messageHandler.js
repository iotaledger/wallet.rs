// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
const utils = require('./utils.js');

console.log(utils);
let { sendMessage, messageHandlerNew } = addon;


const sendMessageAsync = utils.promisify(sendMessage);

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
};

module.exports.MessageHandler = MessageHandler;
