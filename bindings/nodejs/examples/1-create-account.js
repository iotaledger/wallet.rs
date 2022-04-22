/**
 * This example creates a new database and account
 */

require('dotenv').config();
const manager = require('./account-manager');

async function run() {
    const { initLogger } = require('@iota/wallet');

    initLogger({
        color_enabled: true,
        outputs: [
            {
                name: './wallet.log',
                level_filter: 'debug',
            },
        ],
    });

    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        // await manager.storeMnemonic();

        const account = await manager.createAccount({
            alias: 'Alice',
        });
        console.log('Account created:', account);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
