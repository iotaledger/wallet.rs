// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
  try {
    const { AccountManager, SignerType } = require('../lib')
    const manager = new AccountManager({
      storagePath: './test-database'
    })
    manager.setStrongholdPassword('password')
    manager.storeMnemonic(SignerType.Stronghold)

    const account = manager.createAccount({
      clientOptions: { node: 'http://localhost:14265' }
    })
    account.setAlias('banana')
  } finally {
    const fs = require('fs')
    try {
    fs.rmdirSync('./test-database', { recursive: true })
    } catch (e) {
      // ignore it
    }
  }
}

run()