// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    sendMessageAsync,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
} from './bindings';
import type {
    EventType,
    AccountManagerOptions,
    __Message__,
    __AccountMethod__,
    AccountId,
} from '../types';

// The MessageHandler class interacts with messages with the rust bindings.
export class MessageHandler {
    messageHandler: any;

    constructor(options?: AccountManagerOptions) {
        const messageOptions = {
            storagePath: options?.storagePath,
            clientOptions: options?.clientOptions,
            coinType: options?.coinType,
            secretManager: options?.secretManager,
        };

        this.messageHandler = messageHandlerNew(JSON.stringify(messageOptions));
    }

    async sendMessage(message: __Message__): Promise<string> {
        return sendMessageAsync(
            JSON.stringify(message),
            this.messageHandler,
        ).catch((error) => {
            try {
                error = JSON.parse(error).payload;
            } catch (e) {}
            return Promise.reject(error);
        });
    }

    async callAccountMethod(
        accountIndex: AccountId,
        method: __AccountMethod__,
    ): Promise<string> {
        return this.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: accountIndex,
                method,
            },
        });
    }

    listen(
        eventTypes: EventType[],
        callback: (error: Error, result: string) => void,
    ): void {
        return listen(eventTypes, callback, this.messageHandler);
    }

    clearListeners(eventTypes: EventType[]): void {
        return clearListeners(eventTypes, this.messageHandler);
    }

    destroy(): void {
        return destroy(this.messageHandler);
    }
}
