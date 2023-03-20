/**
 * This example shows some events.
 */

const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const account = await manager.getAccount('Alice');
        console.log('Account:', account);

        // Use the Faucet to send testnet tokens to your address:
        console.log(
            'Fill your address with the Faucet: https://faucet.testnet.shimmer.network',
        );

        const callback = function (err, data) {
            console.log('data:', JSON.parse(data));
        };

        // provide event type to filter only for events with this type
        await manager.listen(['TransactionProgress'], callback);

        // send transaction
        await account.sendMicroTransaction([
            {
                address:
                    'rms1qph3f6y3ps7zccucatf70y37kz7udzp94aefg6mzxdgpa5xxerg9u4s0xyz',
                amount: '1000',
            },
        ]);

        // provide event type to remove only event listeners of this type
        manager.clearListeners(['TransactionProgress']);

        const callback2 = function (err, data) {
            console.log('all event data:', JSON.parse(data));
        };
        // provide empty array to listen to all event types
        manager.listen([], callback2);

        // build basic output
        let output = await account.buildBasicOutput({
            amount: '1000000',
            unlockConditions: [
                {
                    type: 0,
                    address: {
                        type: 0,
                        pubKeyHash: `0xc6cf27e8d7c54c420c1315f83567c6bd1b5fad7e9ffa83996680f2aedb2ed0be`,
                    },
                },
            ],
        });
        // send transaction with the output we created
        await account.sendOutputs([output]);

        // provide empty array to clear all listeners for all event types
        setTimeout(() => {
            manager.clearListeners([]);
            console.log('All event listeners removed');
        }, 20000);
    } catch (error) {
        console.log('Error: ', error);
    }

    // Possible Event Types:
    //
    // ConsolidationRequired
    // LedgerAddressGeneration
    // NewOutput
    // SpentOutput
    // TransactionInclusion
    // TransactionProgress
}

run();
