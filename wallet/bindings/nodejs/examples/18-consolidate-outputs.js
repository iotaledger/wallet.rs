/**
 * This example creates a bunch of transactions and then consolidates them
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const alice = await manager.getAccount('Alice');
        const bob = await manager.getAccount('Bob')

        const amount = '300000';
        // Alice needs 3 UTXO's with each at least 300 000 glow to be able to send a transaction to Bob
        for (let i = 0; i < 3; i++) {
            await alice.sync()
            await alice.sendAmount([
                {
                    address: bob.meta.publicAddresses[0].address,
                    amount,
                },
            ]);
        }

        // The timeout is required, since the transaction has to be confirmed before Bob can consolidate
        setTimeout(async () => {
            await bob.sync()
            const resp = await bob.consolidateOutputs(true)
            console.log(resp)
            process.exit(0);
        }, 10000)
        
    } catch (error) {
        console.log('Error: ', error);
        process.exit(1);
    }
}

run();
 