/**
 * This example sends IOTA tokens to an address.
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
        console.log('Account:', account);

        const response = await account.sendAmount([
            {
                //TODO: Replace with the address of your choice!
                address: 'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0',
                amount: '1000000',
            },
        ]);

        console.log(response);

        console.log(
            `Check your block on https://explorer.testnet.shimmer.network/testnet/block/${response.blockId}`,
        );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
