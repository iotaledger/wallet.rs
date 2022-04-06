// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { sendMessage, messageHandlerNew, listen } from './bindings';
import { promisify } from './utils';

console.log('Promiszify', promisify);
console.log('Messate', messageHandlerNew);

const sendMessageAsync = promisify(sendMessage);
const listenAsync = promisify(listen);

export default class MessageHandler {
    messageHandler: any

    constructor(options: any) {
        // each field is stringified before
        let final_options = { storagePath: options.storagePath, clientOptions: JSON.stringify(options.clientOptions), signer: JSON.stringify(options.signer) }
        this.messageHandler = messageHandlerNew(JSON.stringify(final_options));
    }

    async sendMessage(message: unknown): Promise<string> {
        // @ts-ignore
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
    }

    async listen(eventTypes: any, callback: any): Promise<string> {
        // @ts-ignore
        return listenAsync(eventTypes, callback, this.messageHandler);
    }
}
