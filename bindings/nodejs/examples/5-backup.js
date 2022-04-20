/**
 * This example backups your data in a secure file.
 * You can move this file to another app or device and restore it.
 */

require('dotenv').config();

async function run() {
    const { AccountManager } = require('@iota/wallet');

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    try {
        await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const path = await manager.backup('./backup', process.env.SH_PASSWORD);

        console.log('Backup path:', path);
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
