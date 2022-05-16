/**
 * This example creates a new database and account
 */

const unlockAndReturnManager = require('./account-manager');

async function run() {
    try {
        const manager = await unlockAndReturnManager();
        const account = await manager.getAccount('Alice');
        const addressObject = await account.listAddresses();
        console.log('Addresses before:', addressObject);
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
