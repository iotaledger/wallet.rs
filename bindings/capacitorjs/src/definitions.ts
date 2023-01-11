// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type {
  EventType,
  __Message__,
  __AccountMethod__,
  AccountManagerOptions,
} from './types'


export interface IotaWalletMobileTypes {
    initLogger(path: string): Promise<void>;
    messageHandlerNew(messageOptions: AccountManagerOptions): Promise<{
        messageHandler: number;
    }>;
    sendMessage(messageOptions: {
        message: string;
        handler: number;
    }): Promise<{
        result: string;
    }>;
    listen(options: {
        eventTypes: EventType[];
        messageHandler: number;
    }, callback: (message: {
        error: {
            cause: unknown;
        };
        result: string;
    }) => void): Promise<string>;
    clearListeners(options: {
        eventTypes: EventType[];
        messageHandler: number;
    }, callback: (message: {
        error: {
            cause: unknown;
        };
        result: string;
    }) => void): Promise<string>;
    destroy(options: {
        messageHandler: number;
    }): Promise<void>;
}
