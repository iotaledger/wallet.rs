// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { listen, eventListenerNew, removeEventListeners } from './bindings';

export default class EventListener {
  eventListener: any

  constructor(options: any) {
    this.eventListener = eventListenerNew(JSON.stringify(options));
  }

  listen(eventName: string, callback: any) {
    return listen(eventName, this.eventListener, callback);
  }

  removeEventListeners(eventName: string) {
    return removeEventListeners(eventName, this.eventListener);
  }
}
