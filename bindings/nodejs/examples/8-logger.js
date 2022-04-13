/**
 * This example shows some events.
 */

require('dotenv').config();

async function run() {
    const { AccountManager } = require('@iota/wallet');
    const { initLogger } = require('@iota/wallet')

    initLogger({
        color_enabled: true,
        outputs: [{
            name: './wallet.log',
            level_filter: 'debug'
        }]
    })

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
