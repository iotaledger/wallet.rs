/**
 * This example creates a new database and account
 */

require('dotenv').config();

async function run() {
  const { AccountManager } = require('../../lib/index.js');
  const manager = new AccountManager({
    storagePath: './alice-database',
  });
  console.log(process.env.SH_PASSWORD);
  manager.setStrongholdPassword(process.env.SH_PASSWORD);

  const account = manager.getAccount('Alice');

  console.log('Account:', account.alias());

  // Always sync before doing anything with the account
  await account.sync();
  console.log('Syncing...');

  console.log('Available balance', account.balance().available);
}

run();
