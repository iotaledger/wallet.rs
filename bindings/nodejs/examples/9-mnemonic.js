/**
 * This example creates a generates, stores and verifies a mnemonic
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    try {
        const mnemonic = await manager.generateMnemonic();
        console.log('Mnemonic:', mnemonic);

        const verificationResponse = await manager.verifyMnemonic(mnemonic);
        console.log('Verification Response:', verificationResponse);

        // console.log(manager);
        // const result = await manager.storeMnemonic(mnemonic);
        // console.log('Store Mnemonic', result);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit();
}

run();
