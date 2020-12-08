// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
  try {
    const { AccountManager, StorageType } = require('../lib')
    const manager = new AccountManager({
      storagePath: './test-database',
      storageType: StorageType.Stronghold
    })
    manager.setStrongholdPassword('password')

    const mnemonic = 'error morning duty ring fiscal uniform erupt woman push march draw flower pair hello cousin real invest region message chief property vital dismiss moment'
    const account = manager.createAccount({
      mnemonic,
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