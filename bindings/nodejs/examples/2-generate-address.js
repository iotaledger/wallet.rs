/**
 * This example genrates a new address.
 */

require('dotenv').config();

async function run() {
    const { AccountManagerForMessages } = require('@iota/wallet');

    const manager = new AccountManagerForMessages({
        storagePath: './alice-database',
    });

    try {
        // await manager.setStrongholdPassword(process.env.SH_PASSWORD);
        const account = await manager.getAccount('0');
        console.log('Account:', account);

        const address = await account.generateAddresses();
        console.log('New address:', address);

        // Always sync before doing anything with the account
        const synced = await account.sync();
        console.log('Syncing... - ', synced);

        const address2 = await account.generateAddresses();
        console.log('New address:', address2);

        // You can also get the latest unused address:
        // const addressObject = await account.latestAddress();
        // console.log('Address:', addressObject);

        // Use the Chrysalis Faucet to send testnet tokens to your address:
        // console.log("Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/")
    } catch (error) {
        console.log('Error: ' + error);
    }
}

run();
