/**
 * This example will request funds from a faucet
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        const addressObject = (await account.addresses())[0];

        let faucetUrl = "https://faucet.testnet.shimmer.network/api/enqueue"

        const faucetResponse = await account.requestFundsFromFaucet(faucetUrl, addressObject.address);

        console.log(faucetResponse);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
