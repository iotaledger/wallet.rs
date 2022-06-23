/**
 * This example creates a new random mnemonic
 */

const { AccountManager } = require('@iota/wallet');

async function run() {
    try {
        const manager = new AccountManager({
            storagePath: './mnemonic-generation',
            clientOptions: {
                nodes: ['http://localhost:14265'],
            },
            secretManager: "Placeholder",
        });
        
        console.log('Generated mnemonic:', await manager.generateMnemonic());
        // Set generated mnemonic as env variable for MNEMONIC so it can be used in 2-create-account.js

        // delete unecessary db folder again
        require('fs').rmSync('./mnemonic-generation', { recursive: true, force: true });

    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
