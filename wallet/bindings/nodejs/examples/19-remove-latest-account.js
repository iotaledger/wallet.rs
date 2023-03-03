/**
 * This example removes the latest account.
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        
        const manager = await getUnlockedManager();
        console.log('Accounts before:', await manager.getAccounts())
        await manager.removeLatestAccount();
        console.log('Accounts after:', await manager.getAccounts())
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
