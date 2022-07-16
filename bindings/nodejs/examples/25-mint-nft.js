/**
 * This example will mint an NFT
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();

        const response = await account.mintNfts([
            {
                // Hello in bytes
                immutableMetadata: [72, 101, 108, 108, 111],
                metadata: [72, 101, 108, 108, 111],
            }
        ]);

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
