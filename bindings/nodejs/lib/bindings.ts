// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
// @ts-ignore: path is set to match runtime transpiled js path
import { initLogger as internalInitLogger, sendMessage, messageHandlerNew, listen, clearListeners, destroy } from '../../index.node';


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
    internalInitLogger,
    sendMessageAsync,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
};
