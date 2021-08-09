/**
 * This example creates a new database and account
 */

require('dotenv').config()

async function run() {
  const { AccountManager } = require('../../lib/index.js');
  const manager = new AccountManager({
    storagePath: './alice-database',
  });
  try {
    manager.setStrongholdPassword(process.env.SH_PASSWORD);
    let account;
    try {
      account = manager.getAccount("Alice")
    } catch (e) {
      console.log("Couldn't get account, creating a new one")
    }
    // Create account only if it does not already exist
    if (!account) {
      manager.storeMnemonic(1);
      account = manager.createAccount({
        clientOptions: { node: { url: "https://api.lb-0.testnet.chrysalis2.com" }, localPow: true },
        alias: 'Alice',
      });
      console.log('Account created:', account.id())
    }
    let a = await account.sync();
    console.log("synced", a);
  } catch (error) {
    console.log("Error: " + error)
  }
}

run()
