// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
  try {
    const fs = require('fs');
    fs.rmdirSync('./alice-database', { recursive: true });
  } catch (e) {
    // ignore it
  }
  const { AccountManagerForMessages } = require('../lib');

  const manager = new AccountManagerForMessages({
    storagePath: './alice-database',
  });

  // manager.setStrongholdPassword('password');
  // manager.storeMnemonic(SignerType.Stronghold);

  const account = await manager.createAccount({
    clientOptions: {
      node: { url: 'https://api.lb-0.h.chrysalis-devnet.iota.cafe' },
      localPow: true,
    },
    alias: 'Alice',
  });
  console.log('Account created:', account);
  account.setAlias('new alias');

  const savedAccount = manager.getAccount('new alias');
  console.log(savedAccount);

  account.getNodeInfo().then((value) => {
    console.log(value);
  });
}

run();
