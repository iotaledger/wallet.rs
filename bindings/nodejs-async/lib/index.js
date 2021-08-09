// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
const types = require('./main.js')
// const mh = require("./messageHandler.js");
// const el = require("./eventListener.js");
// const am = require("./messages/accountManager.js");
const am = require("./binding/accountManager.js");

let { initLogger } = addon;
// let { MessageHandler } = mh;
// let { EventListener } = el;
let { AccountManager } = am;

initLogger(JSON.stringify({
  color_enabled: true,
  outputs: [{
    name: 'wallet.log',
    level_filter: 'debug'
  }]
}));

module.exports = {
  //   MessageHandler,
  //   EventListener,
  AccountManager,
  initLogger,
  SignerType: {
    Stronghold: 1
  },
};
