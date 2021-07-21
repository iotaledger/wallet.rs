/**
 * This example creates a new database and account
 */

async function run() {
  const { AccountManager } = require('../../lib/index.js');
  const manager = new AccountManager({
    storagePath: './alice-database',
  });
  try {
    manager.setStrongholdPassword("A12345678*");

    let account = manager.getAccount("Alice")
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

    console.log("nodeinfo", await account.getNodeInfo("https://chrysalis-nodes.iota.org", { name: "name", password: "password" }));
  } catch (error) {
    console.log("Error: " + error)
  }
}

run()




// require('dotenv').config()

// async function run() {
//     const { AccountManager, SignerType } = require('@iota/wallet')
//     const manager = new AccountManager({
//         storagePath: './alice-database',
//     })
//     manager.setStrongholdPassword(process.env.SH_PASSWORD)
//     manager.storeMnemonic(SignerType.Stronghold)

//     const account = await manager.createAccount({
//         clientOptions: { node: "https://api.lb-0.testnet.chrysalis2.com", localPow: true },
//         alias: 'Alice',
//     })

//     console.log('Account created:', account.alias())

// }

// run()
