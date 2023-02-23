/**
 * This example will request funds from a faucet
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        const addressObject = (await account.addresses())[0];

        if (!process.env.FAUCET_URL) {
            throw new Error('.env FAUCET_URL is undefined, see .env.example');
        }

        const faucetResponse = await account.requestFundsFromFaucet(process.env.FAUCET_URL, addressObject.address);

        console.log(faucetResponse);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
