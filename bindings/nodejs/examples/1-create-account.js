/**
 * This example creates a new database and account
 */

require('dotenv').config();
const { CoinType } = require('../out/types');
const manager = require('./account-manager');

async function run() {
    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        // await manager.storeMnemonic();

        const account = await manager.createAccount({
            alias: 'Alice',
            coinType: CoinType.Shimmer,
        });
        console.log('Account created:', account);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
