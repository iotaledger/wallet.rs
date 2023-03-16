// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// This file overwrites the `bindings.ts` file from `bindings/nodejs/lib`, to link the Wasm `MessageHandler` interface.
// The rest of the TypeScript definitions are copied as-is to the `out` directory before being compiled.

// Import needs to be in a single line, otherwise it breaks
// prettier-ignore
// @ts-ignore: path is set to match runtime transpiled js path when bundled.
import { initLogger, sendMessageAsync, messageHandlerNew, listen, destroy } from '../wasm/iota_wallet_wasm';

export {
    initLogger as internalInitLogger,
    sendMessageAsync,
    messageHandlerNew,
    listen,
    destroy,
};
