// Copyright 2023 IOTA Stiftung
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

    const account = manager.createAccount({
        ClientOptions: {
            node: 'https://api.testnet.shimmer.network',
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
