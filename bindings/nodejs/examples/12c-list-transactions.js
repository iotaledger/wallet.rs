/**
 * This example lists the transactions of the first account
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        
        await account.sync();
        const transactions = await account.listTransactions()
        console.log('Listing Transactions:', transactions);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
