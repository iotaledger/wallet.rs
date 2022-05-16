/**
 * This example backups your data in a secure file.
 * You can move this file to another app or device and restore it.
 */

require('dotenv').config();
const unlockAndReturnManager = require('./account-manager');

async function run() {
    try {
        const manager = await unlockAndReturnManager();
        const path = await manager.backup('./backup', process.env.SH_PASSWORD);
        console.log('Backup created at:', path);

        await manager.deleteStorage();

        console.log('Successfully created backup');
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
