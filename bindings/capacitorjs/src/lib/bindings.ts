// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler'
import { IotaWalletMobileTypes } from '../definitions'
import { registerPlugin } from '@capacitor/core'

const IotaWalletMobile = registerPlugin<IotaWalletMobileTypes>('IotaWalletMobile')

const {
    initLogger,
    sendMessage,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
} = IotaWalletMobile

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
    })

export {
    initLogger as internalInitLogger,
    sendMessageAsync,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
}
