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
        const addressObject = await account.addresses();
        console.log('Addresses before:', addressObject);

        // syncOnlyMostBasicOutputs if not interested in outputs that are timelocked, 
        // have a storage deposit return or are nft/alias/foundry outputs
        const synced = await account.sync({ syncOnlyMostBasicOutputs: true });
        console.log('Syncing... - ', synced);

        console.log('Available balance', await account.getBalance());

        // Use the Faucet to send testnet tokens to your address:
        console.log("Fill your address with the Faucet: https://faucet.testnet.shimmer.network/")
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
