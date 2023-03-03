// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { EventType } from '../types';
import type { MessageHandler } from './MessageHandler';

// @ts-ignore: path is set to match runtime transpiled js path
import addon = require('../../build/Release/index.node');

const {
    initLogger,
    sendMessage,
    messageHandlerNew,
    listenRust,
    destroy,
} = addon;

const listen = (
    eventTypes: EventType[],
    callback: (error: Error, result: string) => void,
    handler: MessageHandler,
): Promise<void> => {
    listenRust(eventTypes, callback, handler);
    return Promise.resolve();
}

const sendMessageAsync = (
    message: string,
    handler: MessageHandler,
): Promise<string> =>
    new Promise((resolve, reject) => {
        sendMessage(message, handler, (error: Error, result: string) => {
            if (error) {
                reject(error);
            } else {
                resolve(result);
            }
        });
    });

export {
    initLogger as internalInitLogger,
    sendMessageAsync,
    messageHandlerNew,
    listen,
    destroy,
};
