// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
  try {
    const fs = require('fs');
    fs.rmdirSync('./alice-database', { recursive: true });
  } catch (e) {
    // ignore it
  }

  const { AccountManager, SignerType, MessageType } = require('../lib');
  const manager = new AccountManager({
    storagePath: './alice-database',
  });
  manager.setStrongholdPassword('password');
  manager.storeMnemonic(SignerType.Stronghold);

  const account = manager.createAccount({
    clientOptions: {
      node: 'http://api.hornet-3.testnet.chrysalis2.com',
      requestTimeout: {
        secs: 5000,
        nanos: 0,
      },
      apiTimeout: {
        PostMessage: {
          secs: 6000,
          nanos: 0,
        },
      },
    },
  });
  console.log('messages', account.listMessages(0, 0, MessageType.Failed));
  console.log(account.messageCount(MessageType.Failed));
  account.setAlias('new alias');

  const savedAccount = manager.getAccount('new alias');
  console.log(savedAccount);

  account.getNodeInfo().then((value) => {
    console.log(value);
  });
}

run();
