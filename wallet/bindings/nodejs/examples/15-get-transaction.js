/**
 * This example gets a transaction with a given transaction ID.
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('0');
        
        const response = await account.getTransaction('0xafc1205d93655dc1f3561f57291c79a6a50d8f4a95d9328c601d8038c479ecd4')
        console.log(response)
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
