// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
  try {
    const fs = require('fs')
    fs.rmdirSync('./test-database', { recursive: true })
  } catch (e) {
    // ignore it
  }

  const { AccountManager, SignerType, MessageType } = require('../lib')
  const manager = new AccountManager({
    storagePath: './test-database'
  })
  manager.setStrongholdPassword('password')
  manager.storeMnemonic(SignerType.Stronghold)

  const account = manager.createAccount({
    clientOptions: { node: 'http://localhost:14265' }
  })
  console.log('messages', account.listMessages(0, 0, MessageType.Failed))
  account.setAlias('new alias')

  const savedAccount = manager.getAccount('new alias')
  console.log(savedAccount)
}

run()