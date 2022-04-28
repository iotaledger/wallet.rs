// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// @ts-ignore: path is set to match runtime transpiled js path
import addon = require('../../index.node');

import type { LoggerConfig } from '../types';

export * from './AccountManager';
export * from './MessageHandler';
export * from './Account';
export * from '../types';

const initLogger = (config: LoggerConfig) => addon.initLogger(JSON.stringify(config));

export { initLogger };
