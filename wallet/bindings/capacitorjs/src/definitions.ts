// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { EventType, AccountManagerOptions } from './types';
export interface IotaWalletMobileTypes {
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
        error: Error;
        result: string;
    }) => void): Promise<void>;
    destroy(options: {
        messageHandler: number;
    }): Promise<void>;
}
