// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { internalInitLogger } from './lib/bindings'
import type { LoggerConfig } from './types'

export * from './lib/AccountManager'
export * from './lib/MessageHandler'
export * from './lib/Account'
// Moved to definitions.ts, TODO modify tsconfig to adapt it
// export * from './types'

/** Function to create wallet logs */
const initLogger = (config: LoggerConfig) =>
    internalInitLogger(JSON.stringify(config))

export { initLogger }
