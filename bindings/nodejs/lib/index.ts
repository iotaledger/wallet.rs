// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import AccountManager from './AccountManager';
import MessageHandler from './MessageHandler';
import * as Types from './types';

// @ts-ignore
const addon = require('../build/Release/index.node');

const initLogger = (config: any) => addon.initLogger(JSON.stringify(config));

export {
  MessageHandler,
  AccountManager,
  // EventListener,
  initLogger,
  Types
};
