/**
 * This example lists the outputs of the first account
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        
        await account.sync();
        const outputs = await account.outputs()
        console.log('Listing Outputs:', outputs); 
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
