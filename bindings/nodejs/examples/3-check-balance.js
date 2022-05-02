/**
 * This example creates a new database and account
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.balance());

        // Use the Chrysalis Faucet to send testnet tokens to your address:
        // console.log("Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/")
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
