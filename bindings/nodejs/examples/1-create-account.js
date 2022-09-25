/**
 * This example creates a new database and account
 */

require('dotenv').config({ path: '../.env' });
const { AccountManager, CoinType } = require('@iota/wallet');

async function run() {
    try {
        const manager = await createAccountManager();

        const account = await manager.createAccount({
            alias: 'Alice',
        });
        console.log('Account created:', account);

        const secondAccount = await manager.createAccount({
            alias: 'Bob',
        });
        console.log('Account created:', secondAccount);
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
            stronghold: {
                snapshotPath: `./wallet.stronghold`,
                password: `${process.env.SH_PASSWORD}`,
            },
        },
    };

    const manager = new AccountManager(accountManagerOptions);
    await manager.storeMnemonic(process.env.MNEMONIC);
    return manager;
}

run();
