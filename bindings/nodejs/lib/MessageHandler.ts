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
            clientOptions: JSON.stringify(options?.clientOptions),
            coinType: options?.coinType,
            secretManager: JSON.stringify(options?.secretManager),
        };

        this.messageHandler = messageHandlerNew(JSON.stringify(messageOptions));
    }

    async sendMessage(message: __Message__): Promise<string> {
        return sendMessageAsync(JSON.stringify(message), this.messageHandler)
            .catch((error) => {
                const _error = JSON.parse(error);
                return Promise.reject(_error.payload);
            });
    }

    async callAccountMethod(
        accountIndex: AccountId,
        method: __AccountMethod__,
    ): Promise<string> {
        return this.sendMessage({
            cmd: 'CallAccountMethod',
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
