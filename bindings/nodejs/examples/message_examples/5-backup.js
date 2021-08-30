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
    await manager.setStrongholdPassword(process.env.SH_PASSWORD);

    let backup_path = await manager.backup('./backup', process.env.SH_PASSWORD);

    console.log('Backup path:', backup_path);
  } catch (error) {
    console.log('Error: ' + error);
  }
}

run();
