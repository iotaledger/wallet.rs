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
    await manager.storeMnemonic();

    const account = await manager.createAccount({
      // todo replace with https://api.lb-0.h.chrysalis-devnet.iota.cafe when the new faucet is working
      clientOptions: {
        node: { url: 'https://api.lb-0.testnet.chrysalis2.com' },
        localPow: true,
      },
      alias: 'Alice',
    });
    console.log('Account created:', account);
  } catch (error) {
    console.log('Error: ' + error);
  }
}

run();
