/**
 * This example will send native tokens
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        await account.sync();

        // Get a tokenId from your account balance after running example
        // 22-mint-native-tokens.js
        let tokenId =
            '0x087ec7c0a543e60cfc92850ed053d3b323c0d7181e63b24c6ef24dd591814006950100000000';
        // `100` hex encoded
        let tokenAmount = "0x64"

        // Send native tokens with a storage deposit return and an expiraiton of one day
        // This means that the receiver has to claim the output in time (can be done with 21-claim-outputs.js),
        // where the storage deposit of the output is returned, or if not, the sender gets full control back after one day passed.
        const response = await account.sendNativeTokens([
            {
                //TODO: Replace with the address of your choice!
                address: 'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0',
                nativeTokens: [[tokenId, tokenAmount]],
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
