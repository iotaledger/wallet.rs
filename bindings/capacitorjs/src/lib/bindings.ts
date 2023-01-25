// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { IotaWalletMobile } from '../index'

const {
    initLogger,
    sendMessage,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
} = IotaWalletMobile

const sendMessageAsync = async (message: string, handler: number): Promise<string> => {
    const { result } = await sendMessage({ message, handler })
    return result
}

export {
    IotaWalletMobile,
    initLogger as internalInitLogger,
    sendMessageAsync,
    messageHandlerNew,
    listen,
    clearListeners,
    destroy,
}
