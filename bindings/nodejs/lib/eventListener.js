// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
let { listen, eventListenerNew, removeEventListeners } = addon;

class EventListener {
  constructor(options) {
    console.log("EventListener constructor called.");
    this.eventListener = eventListenerNew(JSON.stringify(options));
  }

  listen(eventName, callback) {
    console.log("listen called.");
    console.log(eventName);
    return listen(eventName, this.eventListener, callback);
  }

  removeEventListeners(eventName) {
    console.log("removeEventListeners called.");
    console.log(eventName);
    return removeEventListeners(eventName, this.eventListener, callback);
  }
};

module.exports.EventListener = EventListener;
