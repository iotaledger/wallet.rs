/**
 * This example shows how to use events.
 */

require('dotenv').config();

async function run() {
    const { AccountManager } = require('@iota/wallet');

    const manager = new AccountManager({
        storagePath: './alice-database',
    });

    manager.setStrongholdPassword(process.env.SH_PASSWORD);

    const account = manager.getAccount('Alice');
    console.log('Account:', account.alias());

    // Always sync before doing anything with the account
    await account.sync();
    console.log('Syncing...');
    // let address = account.generateAddress()

    // get latest address
    let addressObject = account.latestAddress();

    console.log('Address:', addressObject.address);

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log(
        'Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/',
    );

    const callback = function (err, data) {
        if (err) {
            console.error(err);
        } else {
            console.log('Data:', data);
        }
    };

    manager.listen('BalanceChange', callback);

    // Event listeners would be removed after 30 seconds.
    setTimeout(() => {
        manager.removeEventListeners('BalanceChange');
        console.log('Event listeners removed');

        // Exit the process
        process.exit(0);
    }, 30000);

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
