/**
 * This example lists the pending transactions of the first account
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        const bob = await manager.getAccount('Bob');

        const { address } = bob.meta.publicAddresses[0];
        const amount = '1000000';

        await account.sendAmount([
            {
                address,
                amount,
            },
        ]);
        const pendingTransactions = await account.pendingTransactions()
        console.log('Listing Pending Transactions:', pendingTransactions[0]?.payload);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
