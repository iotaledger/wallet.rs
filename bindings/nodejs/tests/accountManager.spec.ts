// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const { AccountManager, CoinType } = require('../lib');

describe('AccountManager', () => {
    it('create account', async () => {
        let storagePath = "test-create-account";
        removeDir(storagePath)

        const accountManagerOptions = {
            storagePath: './test-create-account',
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-create-account/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const accountManager = new AccountManager(accountManagerOptions);
        await accountManager.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const account = await accountManager.createAccount({
            alias: 'Alice',
        });

        expect(account.getMetadata().index).toStrictEqual(0);

        await accountManager.destroy()
        removeDir(storagePath)
    });

    it('generate address', async () => {
        let storagePath = "test-generate-address";
        removeDir(storagePath)

        const accountManagerOptions = {
            storagePath: './test-generate-address',
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-generate-address/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const accountManager = new AccountManager(accountManagerOptions);
        await accountManager.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const address = await accountManager.generateAddress(
            0,
            false,
            0,
            { ledgerNanoPrompt: false },
            "rms"
        );

        expect(address).toStrictEqual("rms1qpqzgvcehafmlxh87zrf9w8ck8q2kw5070ztf68ylhzk89en9a4fy5jqrg8");

        const anotherAddress = await accountManager.generateAddress(
            10,
            true,
            10,
            { ledgerNanoPrompt: false },
            "tst"
        );

        expect(anotherAddress).toStrictEqual("tst1qzp37j45rkfmqn05fapq66vyw0vkmz5zqhmeuey5fked0wt4ry43jeqp2wv");

        await accountManager.destroy()
        removeDir(storagePath)
    });
})

function removeDir(storagePath: string) {
    const fs = require('fs');
    fs.rmSync(storagePath, { recursive: true, force: true });
}
