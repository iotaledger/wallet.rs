/**
 * This example backups your data in a secure file.
 * You can move this file to another app or device and restore it.
 */

require('dotenv').config();
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const path = await manager.backup('./backup', process.env.SH_PASSWORD);
        console.log('Backup created at:', path);

        await manager.deleteAccountsAndDatabase();

        console.log('Successfully created backup');
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
