// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// import type { MessageHandler } from './MessageHandler'
// import { IotaWalletMobileTypes } from "@iota/wallet-mobile"
// import { registerPlugin } from '@capacitor/core'

// const IotaWalletMobile = registerPlugin<IotaWalletMobileTypes>('IotaWalletMobile')
import { IotaWalletMobile } from '../index'
// const IotaWalletMobile: IotaWalletMobileTypes

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
