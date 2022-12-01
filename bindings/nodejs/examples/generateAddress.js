/**
 * This example generates an address without storing it.
 */

require('dotenv').config({ path: '../.env' });
const { AccountManager, CoinType } = require('@iota/wallet');

async function run() {
    try {
        const manager = await createAccountManager();

        const address = await manager.generateAddress(
            0,
            false,
            0,
            { ledgerNanoPrompt: false },
            "tst"
        );
        console.log('Address:', address);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

async function createAccountManager() {
    const accountManagerOptions = {
        storagePath: './alice-database',
        clientOptions: {
            nodes: ['https://api.testnet.shimmer.network'],
            localPow: true,
        },
        coinType: CoinType.Shimmer,
        secretManager: {
            Stronghold: {
                snapshotPath: `./wallet.stronghold`,
                password: `${process.env.SH_PASSWORD}`,
            },
        },
    };

    const manager = new AccountManager(accountManagerOptions);
    // await manager.storeMnemonic(process.env.MNEMONIC);
    return manager;
}

run();
