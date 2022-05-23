// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    sendMessageAsync,
    messageHandlerNew,
    listen,
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
            secretManager: JSON.stringify(options?.secretManager),
        };

        this.messageHandler = messageHandlerNew(JSON.stringify(messageOptions));
    }

    async sendMessage(message: __Message__): Promise<string> {
        return sendMessageAsync(JSON.stringify(message), this.messageHandler);
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

    destroy(): void {
        return destroy(this.messageHandler);
    }
}
