// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../../index.node');
const utils = require('../utils.js');

let { sendMessage, messageHandlerNew, listen, } = addon;

const sendMessageAsync = utils.promisify(sendMessage);
const listenAsync = utils.promisify(listen);

class MessageHandler {
    constructor(options) {
        // each field is stringified before
        let final_options = {storagePath: options.storagePath, clientOptions: JSON.stringify(options.clientOptions), signer: JSON.stringify(options.signer)}
        this.messageHandler = messageHandlerNew(JSON.stringify(final_options));
    }

    async sendMessage(message) {
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
    }

    async listen(eventTypes, callback) {
        return listenAsync(eventTypes, callback, this.messageHandler);
    }
}

module.exports.MessageHandler = MessageHandler;
