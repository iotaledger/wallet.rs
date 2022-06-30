/**
 * This example gets the balance of an account
 */

require('dotenv').config();
const { AccountManager } = require('@iota/wallet');

async function run() {
    try {
        const manager = new AccountManager({
            storagePath: './alice-database',
        });

        const account = await manager.getAccount('Alice');
        const addressObject = await account.listAddresses();
        console.log('Addresses before:', addressObject);

        // Always sync before calling getBalance()
        // todo: also ignore outputs with sdr/expiration/timelock?
        const synced = await account.sync({syncAliasesAndNfts: false});
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.getBalance());

        // Use the Chrysalis Faucet to send testnet tokens to your address:
        // console.log("Fill your address with the Faucet: https://faucet.chrysalis-devnet.iota.cafe/")
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
