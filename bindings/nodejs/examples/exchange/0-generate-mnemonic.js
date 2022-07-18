/**
 * This example generates a new random mnemonic
 */

const { AccountManager, CoinType } = require('@iota/wallet');

async function run() {
    try {
        const manager = new AccountManager({
            storagePath: './mnemonic-generation',
            clientOptions: {
                nodes: ['http://localhost:14265'],
            },
            coinType: CoinType.Shimmer,
            // Placeholder can't be used for address generation or signing, but we can use it since we only want to generate a mnemonic
            secretManager: "Placeholder",
        });
        
        console.log('Generated mnemonic:', await manager.generateMnemonic());
        // Set generated mnemonic as env variable for MNEMONIC so it can be used in 2-create-account.js

        // delete unecessary db folder again
        require('fs').rmSync('./mnemonic-generation', { recursive: true, force: true });

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
