/**
 * This example generates a new address.
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');
        console.log('Account:', account);

        const address = await account.generateAddress();
        console.log('New address:', address);

        // It's also possible to generate multiple addresses
        // const addresses = await account.generateAddresses(2);
        // console.log('New addresses:', addresses);

        // Use the Faucet to send testnet tokens to your address:
        console.log("Fill your address with the Faucet: https://faucet.testnet.shimmer.network")
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
