// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
let { listen, eventListenerNew, removeEventListeners } = addon;
class EventListener {
  constructor(options) {
    this.eventListener = eventListenerNew(JSON.stringify(options));
  }

  listen(eventName, callback) {
    return listen(eventName, this.eventListener, callback);
  }

  removeEventListeners(eventName) {
    return removeEventListenersAsync(eventName, this.eventListener);
  }
};

module.exports.EventListener = EventListener;
