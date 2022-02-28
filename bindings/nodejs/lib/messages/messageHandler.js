// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');

let { sendMessage, messageHandlerNew, listen, test } = addon;

const sendMessageAsync = utils.promisify(sendMessage);
const listenAsync = utils.promisify(listen);
const testAsync = utils.promisify(test);

class MessageHandler {
    constructor(options) {
        this.messageHandler = messageHandlerNew(JSON.stringify(options));
    }

    async sendMessage(message) {
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
    }

    async listen(eventTypes, callback) {
        return listenAsync(eventTypes, callback, this.messageHandler);
    }
}

module.exports.MessageHandler = MessageHandler;
