/**
 * This example creates an address for an account
 */

require('dotenv').config();
const { AccountManager } = require('@iota/wallet');

async function run() {
    try {
        const manager = new AccountManager({
            storagePath: './alice-database',
        });

        await manager.setStrongholdPassword(`${process.env.SH_PASSWORD}`)
        
        const account = await manager.getAccount('Alice');

        const address = await account.generateAddress()

        console.log('Address generated:', address);

    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
