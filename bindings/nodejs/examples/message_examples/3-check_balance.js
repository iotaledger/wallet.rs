/**
 * This example creates a new database and account
 */

async function run() {
  const { AccountManagerForMessages } = require('../../lib/index.js');

  const manager = new AccountManagerForMessages({
    storagePath: './alice-database',
  });

  try {
    await manager.setStrongholdPassword('A12345678*');

    const account = await manager.getAccount('Alice');
    console.log('Account:', account);

    // Always sync before doing anything with the account
    const synced = await account.sync();
    console.log('Syncing... - ' + synced);

    console.log('Available balance', await account.balance());

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    // console.log("Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/")
  } catch (error) {
    console.log('Error: ' + error);
  }
}

run();
