/**
 * This example sends IOTA tokens to an address.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        //TODO: test this once outputs can be conveniently built
        // const response = await account.SendTransaction([
        //     {
        //         address,
        //         amount,
        //     },
        // ]);

        // console.log(response);

        // console.log(
        //     `Check your block on http://localhost:14265/api/v2/blocks/${response.blockId}`,
        // );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
