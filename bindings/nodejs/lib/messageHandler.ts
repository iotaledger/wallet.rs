// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { sendMessageAsync, messageHandlerNew, listen } from './bindings';
import type { EventType, AccountManagerOptions, __SendMessagePayload__ } from './types';

/**
 * Message Handler class
 */
export default class MessageHandler {
    messageHandler: any

    /**
    * Creates a new instance of Account Manager
    * 
    * @param {AccountManagerOptions} options 
    */
    constructor(options: AccountManagerOptions) {
        const messageOptions = {
            storagePath: options?.storagePath,
            clientOptions: JSON.stringify(options?.clientOptions),
            signer: JSON.stringify(options?.signer)
        }

        this.messageHandler = messageHandlerNew(JSON.stringify(messageOptions));
    }

    /**
     * Sends a message to bindings
     * 
     * @param {Mess} message 
     * 
     * @returns {Promise<string>}
     */
    async sendMessage(message: __SendMessagePayload__): Promise<string> {
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
    }

    /**
     * Listen to events supported by bindings
     * 
     * @param {EventType[]} eventTypes 
     * @param {Function} callback 
     * 
     * @returns {void}
     */
    listen(eventTypes: EventType[], callback: (error: Error, result: string) => void): void {
        return listen(eventTypes, callback, this.messageHandler);
    }
}
