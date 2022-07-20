/**
 * This example will send an NFT
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const { initLogger } = require('@iota/wallet');
    initLogger({
        name: './wallet.log',
        levelFilter: 'debug',
        targetExclusions: ["h2", "hyper", "rustls"]
    });
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();

        // Send the full NFT output to the specified address
        const response = await account.sendNft([{
            //TODO: Replace with the address of your choice!
            address: 'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0',
            //TODO: Replace with an NFT id from your account, you can mint one with `25-mint-nft.js`.
            nftId: '0x09aa7871e126cc41f1f3784a479a5dd5f23e4dd8b97e932a001e77a11ad42f0c',
        }]);

        console.log(response);

        console.log(
            `Check your block on http://localhost:14265/api/core/v2/blocks/${response.blockId}`,
        );
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
