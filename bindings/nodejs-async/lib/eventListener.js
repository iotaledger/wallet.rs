// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
const utils = require('./utils.js');
console.log(utils);
let { listen, eventListenerNew, removeEventListeners } = addon;

const listenAsync = utils.promisify(listen);
const removeEventListenersAsync = utils.promisify(removeEventListeners);
class EventListener {
  constructor(options) {
    console.log("EventListener constructor called.");
    this.eventListener = eventListenerNew(JSON.stringify(options));
  }

  async listen(eventName) {
    console.log("listen called.");
    console.log(eventName);
    return listenAsync(eventName, this.eventListener);
  }

  async removeEventListeners(eventName) {
    console.log("removeEventListeners called.");
    console.log(eventName);
    return removeEventListenersAsync(eventName, this.eventListener);
  }
};

module.exports.EventListener = EventListener;
