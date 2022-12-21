// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// import { PluginListenerHandle } from "@capacitor/core"
// import type { MessageHandler } from "./lib/MessageHandler"
import type {
  EventType,
  __Message__,
  __AccountMethod__,
  // AccountManagerOptions,
} from './types'


export interface IotaWalletMobileTypes {
  initLogger(path: string): Promise<void>
  
  messageHandlerNew(messageOptions: { 
    storagePath: string 
  }): Promise<{ messageHandler: number }>
  
  sendMessage(messageOptions: {
    message: string, 
    handler : number, 
  }): Promise<{ result: string }>
  
  listen(
    options: {
      eventTypes: EventType[];
      messageHandler: number;
    },
    callback: (message: {
      error: {
        cause: unknown;
      };
      result: string;
    }) => void
  ): Promise<string>;
  
  clearListeners(options: {
    eventTypes: EventType[],
    messageHandler: number
  }): Promise<{ result: string }>
  
  destroy(options: { 
    messageHandler: number 
  }): Promise<void>;
}
