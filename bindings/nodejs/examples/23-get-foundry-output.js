/**
 * This example will get a foundry output by native token id. It will first
 * try to get the foundry from the account, and if it isn't in the account
 * it will try to get it from the node
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        // Get a tokenId from your account balance after running example
        // 22-mint-native-tokens.js
        let tokenId =
            '0x08a08898630b0a76a455c85c5e8d13ec56fa905c3bf1b619625c5dab45ddf02f620100000000';

        let foundryOutput = await account.getFoundryOutput(tokenId);
        console.log('Foundry output from account:\n', foundryOutput);

        // Use a different account
        const secondAccount = await manager.getAccount('1');

        // Retrieve the foundry output again, this time the output is not
        // in the account so it will be retrieved from the node
        console.log(
            'Foundry output from node:\n',
            await secondAccount.getFoundryOutput(tokenId),
        );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
