// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../build/Release/index.node');
const mh = require('./messages/messageHandler.js');
const el = require('./eventListener.js');
const amm = require('./messages/accountManager.js');
const am = require('./binding/accountManager.js');
const { RemainderValueStrategy, OutputKind } = require('./utils.js');

let { initLogger } = addon;
let { MessageHandler } = mh;
let { EventListener } = el;
let { AccountManager } = am;
let { AccountManagerForMessages } = amm;

// initLogger(JSON.stringify({
//   color_enabled: true,
//   outputs: [{
//     name: 'wallet.log',
//     level_filter: 'debug'
//   }]
// }));

module.exports = {
  MessageHandler,
  AccountManagerForMessages,
  EventListener,
  AccountManager,
  RemainderValueStrategy,
  OutputKind,
  initLogger: (config) => initLogger(JSON.stringify(config)),
  SignerType: {
    Stronghold: 1,
  },
  MessageType: {
    Received: 1,
    Sent: 2,
    Failed: 3,
    Unconfirmed: 4,
    Value: 5,
    Confirmed: 6,
  },
};
