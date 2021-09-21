/**
 * This example shows some events.
 */

require('dotenv').config();

async function run() {
    const { AccountManagerForMessages } = require('@iota/wallet');

    const manager = new AccountManagerForMessages({
        storagePath: './alice-database',
    });

    try {
        await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        // You can also get the latest unused address:
        const addressObject = await account.latestAddress();
        console.log('Address:', addressObject);

        // Use the Chrysalis Faucet to send testnet tokens to your address:
        console.log(
            'Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/',
        );

        const callback = function (err, data) {
            console.log('data:', data);
        };
        manager.listen('BalanceChange', callback);

        setTimeout(() => {
            manager.removeEventListeners('BalanceChange');
            console.log('event listeners removed');
        }, 300000);
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
