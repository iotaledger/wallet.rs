/**
 * This example backups your data in a secure file.
 * You can move this file to another app or device and restore it.
 */

async function run() {
  const { AccountManagerForMessages } = require('../../lib/index.js');
  const manager = new AccountManagerForMessages({
    storagePath: './alice-database',
  });
  try {
    await manager.setStrongholdPassword('A12345678*');

    let backup_path = await manager.backup('./backup', 'A12345678*');

    console.log('Backup path:', backup_path);
  } catch (error) {
    console.log('Error: ' + error);
  }
}

run();
