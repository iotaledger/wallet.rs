// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

async function run() {
    try {
        const fs = require('fs');
        fs.rmdirSync('./alice-database', { recursive: true });
    } catch (e) {
        // ignore it
    }
    const { AccountManager, CoinType } = require('../node/lib');

    const manager = new AccountManager({
        storagePath: './alice-database',
        coinType: CoinType.Shimmer,
        clientOptions: {
            nodes: ['https://api.testnet.shimmer.network'],
        },
        secretManager: {
            mnemonic: "inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak",
        },
    });

    const account = await manager.createAccount({
        alias: 'Alice',
    });
    console.log('Account created:', account);
    account.setAlias('new alias');

    const balance = await account.sync();
    console.log(balance);

    manager.getNodeInfo().then((value) => {
        console.log(value);
    });

}

run();
