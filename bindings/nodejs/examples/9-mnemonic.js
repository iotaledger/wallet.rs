/**
 * This example creates a generates, stores and verifies a mnemonic
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    try {
        const mnemonic = await manager.generateMnemonic();
        console.log('Mnemonic:', mnemonic);

        await manager.verifyMnemonic(mnemonic);

        await manager.storeMnemonic(mnemonic);
        console.log('Mnemonic successfully stored!');
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit();
}

run();
