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
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);

        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        // You can also get the latest unused address:
        // const addressObject = await account.latestAddress();
        // console.log('Address:', addressObject);

        // Use the Chrysalis Faucet to send testnet tokens to your address:
        console.log(
            'Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/',
        );

        const callback = function (err, data) {
            console.log('all event data:', JSON.parse(data));
        };
        // empty array listens to all event types
        manager.listen([], callback);

        const callback2 = function (err, data) {
            console.log('data:', JSON.parse(data));
        };
        // provide event type to filter only for events with this type
        manager.listen(['BalanceChange'], callback2);

        const address = await account.generateAddresses();
        console.log('New address:', address);

        setTimeout(() => {
            manager.removeEventListeners('BalanceChange');
            console.log('event listeners removed');
        }, 300000);
    } catch (error) {
        console.log('Error: ' + error);
    }

    // Possible Event Types:
    //
    // BalanceChange
    // TransactionInclusion
    // TransferProgress
    // ConsolidationRequired
    // LedgerAddressGeneration
}

run();
