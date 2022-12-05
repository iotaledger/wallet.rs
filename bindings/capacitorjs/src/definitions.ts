// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { PluginListenerHandle } from "@capacitor/core"
import { MessageHandler } from "./lib/MessageHandler"
import type {
  EventType,
  AccountManagerOptions,
  __Message__,
  __AccountMethod__,
} from './types'

export * from './types'
export interface IotaWalletMobileTypes {
  initLogger(path: string): Promise<void>
  messageHandlerNew(messageOptions: string): Promise<void>
  clearListeners(
    eventTypes: EventType[],
    messageHandler: any
  ): void
  initialize(options: AccountManagerOptions): Promise<void>
  listen(
    eventTypes: EventType[], 
    callback: (error: Error, result: string) => void,
    messageHandler: any
  ): void
  destroy(messageHandler: any): void
  sendMessage(
    message: string, 
    handler :MessageHandler, 
    callback: (error: Error, result: string) => void
  ): PluginListenerHandle
}
