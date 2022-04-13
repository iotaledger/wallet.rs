// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

export * from './AccountManager';
export * from './MessageHandler';
export * from './Account';
export * from '../types';

// @ts-ignore
const addon = require('../../build/Release/index.node');

const initLogger = (config: any) => addon.initLogger(JSON.stringify(config));

export { initLogger };
