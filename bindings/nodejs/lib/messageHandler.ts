// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { sendMessage, messageHandlerNew } from './bindings';
import { promisify } from './utils';

console.log('Promiszify', promisify);
console.log('Messate', messageHandlerNew);

const sendMessageAsync = promisify(sendMessage);

export default class MessageHandler {
    messageHandler: any

    constructor(options: any) {
        this.messageHandler = messageHandlerNew(JSON.stringify(options));
    }

    async sendMessage(message: unknown): Promise<string> {
        // @ts-ignore
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
    }
}
