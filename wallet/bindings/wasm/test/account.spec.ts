// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import fs from 'fs';
import {
    Account,
    AccountBalance,
    AccountManager,
    CoinType,
    NodeInfoWrapper,
} from '../node/lib';

async function run() {
    try {
        fs.rmdirSync('./test-alice-database', { recursive: true });
    } catch (e) {
        // ignore it
    }

    const manager = new AccountManager({
        storagePath: './test-alice-database',
        coinType: CoinType.Shimmer,
        clientOptions: {
            nodes: ['http://localhost:14265'],
        },
        secretManager: {
            mnemonic:
                'inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak',
        },
    });

    const account: Account = await manager.createAccount({
        alias: 'Alice',
    });

    expect(account.getMetadata().alias).toBe('Alice');

    const balance: AccountBalance = await account.sync();
    expect(balance.baseCoin.available).not.toBeNaN();

    await account.setAlias('new alias');
    const savedAccount: Account = await manager.getAccount('new alias');
    expect(savedAccount).not.toBeNull();

    manager.getNodeInfo().then((value: NodeInfoWrapper) => {
        expect(value.url).toBe('http://localhost:14265');
    });
}

// Tests that do not require a node
describe('Wallet methods', () => {
    it('generate mnemonic', async () => {
        const manager = new AccountManager({
            storagePath: './test-generate-mnemonic',
            coinType: CoinType.Shimmer,
            clientOptions: {
                nodes: [],
            },
            secretManager: 'placeholder'
        });

        const mnemonic = await manager.generateMnemonic();
        expect(mnemonic.split(' ').length).toBe(24);
    });
});

// Tests requiring a local node
describe.skip('local node tests', () => {
    jest.setTimeout(10000);
    it('account', async () => {
        await run();
    });
});
