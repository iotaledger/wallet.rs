// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../build/Release/index.node');
const { listen, eventListenerNew, removeEventListeners } = addon;

class EventListener {
  constructor(options) {
    this.eventListener = eventListenerNew(JSON.stringify(options));
  }

  listen(eventName, callback) {
    return listen(eventName, this.eventListener, callback);
  }

  removeEventListeners(eventName) {
    return removeEventListeners(eventName, this.eventListener);
  }
}

module.exports.EventListener = EventListener;
