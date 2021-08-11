// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
// const mh = require("./messages/messageHandler.js");
const el = require("./eventListener.js");
// const am = require("./messages/accountManager.js");
const am = require("./binding/accountManager.js");

let { initLogger } = addon;
// let { MessageHandler } = mh;
let { EventListener } = el;
let { AccountManager } = am;

// todo remove this
initLogger(JSON.stringify({
  color_enabled: true,
  outputs: [{
    name: 'wallet.log',
    level_filter: 'debug'
  }]
}));

function addEventListener(name, callback) {
    eventListener = new EventListener();
    eventListener.listen(name, callback);
    return eventListener;
}

module.exports = {
//   MessageHandler,
  EventListener,
  AccountManager,
  addEventListener,
  initLogger: config => initLogger(JSON.stringify(config)),
  SignerType: {
    Stronghold: 1
  },
};
