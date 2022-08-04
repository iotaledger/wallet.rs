/**
 * This example will burn an NFT
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

        //TODO: Replace with an NFT id from your account, you can mint one with `25-mint-nft.js`.
        const response = await account.burnNft('0xeb7a7f6b4b8f932ed0d60d5a6018cb51dfa53af1173f9ca8944d1ab49772dd2b');

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
