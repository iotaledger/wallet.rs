/**
 * This example shows some events.
 */

async function run() {
  const {
    AccountManagerForMessages,
    EventListener,
  } = require('../../lib/index.js');
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

    // You can also get the latest unused address:
    const addressObject = await account.latestAddress();
    console.log('Address:', addressObject);

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log(
      'Fill your address with the Faucet: https://faucet.testnet.chrysalis2.com/',
    );

    const callback = function (err, data) {
      console.log('data:', data);
    };
    let el = new EventListener();
    console.log(el);
    el.listen('BalanceChange', callback);
  } catch (error) {
    console.log('Error: ' + error);
  }

  // Possible Event Types:
  //
  // ErrorThrown
  // BalanceChange
  // NewTransaction
  // ConfirmationStateChange
  // Reattachment
  // Broadcast
}

run();
