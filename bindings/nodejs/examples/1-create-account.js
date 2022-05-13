/**
 * This example creates a new database and account
 */

require('dotenv').config();
const { CoinType } = require('../out/types');
const manager = require('./account-manager');

async function run() {
    try {
        await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        // The coin type only needs to be set on the first account
        const account = await manager.createAccount({
            alias: 'Alice',
            coinType: CoinType.IOTA,
        });
        console.log('Account created:', account);

        const secondAccount = await manager.createAccount({
            alias: 'Bob',
        });
        console.log('Account created:', secondAccount);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
