/**
 * This example generates a new address.
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

    const address = account.generateAddress();
    console.log('New address:', address);

    // You can also get the latest unused address:
    const addressObject = account.latestAddress();
    console.log('Address:', addressObject.address);

    // Use the Chrysalis Faucet to send testnet tokens to your address:
    console.log(
        'Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/',
    );

    const addresses = account.listAddresses();
    console.log('Addresses:', addresses);
}

run();
