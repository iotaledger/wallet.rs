/**
 * This example initializes the logger.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    const { initLogger } = require('@iota/wallet');
    initLogger({
        name: './wallet.log',
        levelFilter: 'debug',
        targetExclusions: ["h2", "hyper", "rustls"]
    });

    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        console.log('Account:', account);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
