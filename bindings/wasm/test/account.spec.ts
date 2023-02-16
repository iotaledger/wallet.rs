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
        fs.rmdirSync('./alice-database', { recursive: true });
    } catch (e) {
        // ignore it
    }

    const manager = new AccountManager({
        storagePath: './alice-database',
        coinType: CoinType.Shimmer,
        clientOptions: {
            nodes: ['https://api.testnet.shimmer.network'],
        },
        secretManager: {
            mnemonic:
                'inhale gorilla deny three celery song category owner lottery rent author wealth penalty crawl hobby obtain glad warm early rain clutch slab august bleak',
        },
    });

    const account: Account = await manager.createAccount({
        alias: 'Alice',
    });

    expect(account.alias()).toBe('Alice');

    account.setAlias('new alias');
    expect(account.alias()).toBe('new alias');

    const balance: AccountBalance = await account.sync();
    expect(balance.baseCoin.available).toBe('0');

    const savedAccount: Account = await manager.getAccount('new alias');
    expect(savedAccount).not.toBeNull();

    manager.getNodeInfo().then((value: NodeInfoWrapper) => {
        expect(value.url).toBe('https://api.testnet.shimmer.network');
    });
}

describe('Wallet methods', () => {
    it('account', async () => {
        run();
    });
});
