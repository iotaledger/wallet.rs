/**
 * This example generates, stores and verifies a mnemonic
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const mnemonic = await manager.generateMnemonic();
        console.log('Mnemonic:', mnemonic);

        await manager.verifyMnemonic(mnemonic);

        await manager.storeMnemonic(mnemonic);
        console.log('Mnemonic successfully stored!');
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit();
}

run();
