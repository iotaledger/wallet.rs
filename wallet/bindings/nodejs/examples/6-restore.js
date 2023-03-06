/**
 * This example restores a secured backup file.
 */
const path = require('path')
require('dotenv').config({ path: path.resolve(__dirname, '.env') });
const { AccountManager, CoinType } = require('@iota/wallet');
async function run() {
    try {
        const accountManagerOptions = {
            storagePath: './restore-database',
            clientOptions: {
                nodes: ['https://api.testnet.shimmer.network'],
                localPow: true,
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                stronghold: { snapshotPath: 'restore.stronghold'},
            },
        };

        const manager = new AccountManager(accountManagerOptions);

        // Add the path to the file from example 5-backup.js
        // for example: ./backup/2021-02-12T01-23-11-iota-wallet-backup-wallet.stronghold
        const path = './backup'; //"input your backup file"

        await manager.restoreBackup(path, process.env.SH_PASSWORD);
        const account = await manager.getAccount('Alice');
        console.log('Account:', account.getMetadata().alias);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
