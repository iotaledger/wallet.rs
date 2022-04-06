// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// @ts-ignore
const addon = require('../build/Release/index.node');

import AccountManagerForMessages from './accountManager';
import MessageHandler from './messageHandler';
import { remainderValueStrategy, OutputKind } from './utils'

const initLogger = (config: any) => addon.initLogger(JSON.stringify(config));

export {
  MessageHandler,
  AccountManagerForMessages,
  // EventListener,
  remainderValueStrategy,
  OutputKind,
  initLogger,
};

