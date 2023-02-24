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
            `Check your block on ${process.env.NODE_URL}/api/core/v2/blocks/${response.blockId}`,
        );

        // To send an NFT with expiration unlock condition prepareOutput() can be used like this:
        // const output = await account.prepareOutput({
        //     recipientAddress: 'rms1qz6aj69rumk3qu0ra5ag6p6kk8ga3j8rfjlaym3wefugs3mmxgzfwa6kw3l',
        //     amount: "47000",
        //     unlocks: {
        //         expirationUnixTime: 1677065933
        //     },
        //     assets: {
        //         nftId: '0x447b20b81e2311a6c16a32eaeda2f2f2472c4b43ed4ffc80a0c0f850130fc4bb',
        //     },
        //     storageDeposit: { returnStrategy: 'Gift' }
        // });

        // const transaction = await account.sendOutputs([output]);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
