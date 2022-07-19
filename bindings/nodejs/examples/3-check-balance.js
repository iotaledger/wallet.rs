/**
 * This example gets the balance for an account
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        const addressObject = await account.listAddresses();
        console.log('Addresses before:', addressObject);

        // Always sync before calling getBalance()
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.getBalance());

        // Use the Faucet to send testnet tokens to your address:
        console.log("Fill your address with the Faucet:  https://faucet.testnet.shimmer.network")
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
