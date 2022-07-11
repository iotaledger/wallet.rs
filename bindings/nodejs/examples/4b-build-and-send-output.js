/**
 * This example sends IOTA tokens to an address.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        
        // The address has to be specified in Ed25519 format
        const output = await account.buildBasicOutput({
            amount: '2000000',
            unlockConditions: [{
                type: 0,
                address: {
                    type: 0,
                    pubKeyHash:`0xc6cf27e8d7c54c420c1315f83567c6bd1b5fad7e9ffa83996680f2aedb2ed0be`
                },
            }]
        })

        console.log('Output built:', output)

        const response = await account.sendOutputs([output]);

        console.log(response);

        // console.log(
        //     `Check your block on http://localhost:14265/api/core/v2/blocks/${response.blockId}`,
        // );
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

run();
