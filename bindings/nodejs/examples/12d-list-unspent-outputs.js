/**
 * This example lists the unspent outputs of the first account
 */
const unlockAndReturnManager = require('./account-manager');

async function run() {
    try {
        const manager = await unlockAndReturnManager();
        const account = await manager.getAccount('Alice');
        
        await account.sync();
        const outputs = await account.listUnspentOutputs()
        console.log('Listing Unspent Outputs:', outputs);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
