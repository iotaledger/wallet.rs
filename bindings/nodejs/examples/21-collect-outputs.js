/**
 * This example sends IOTA tokens with an expiration date and collects them.
 */

const getUnlockedManager = require('./account-manager');

let manager
let bob
async function run() {
    try {
        manager = await getUnlockedManager();
        const alice = await manager.getAccount('Alice');
        bob = await manager.getAccount('Bob');

        manager.listen(['NewOutput'], handleNewOutput)

        const recipientAddress = bob.meta.publicAddresses[0].address
        const amount = '1000000';

        const outputData = await alice.prepareOutput(
            {
                recipientAddress,
                amount,
                unlocks: {
                    expiration: {
                        unixTime: new Date().getSeconds() + 15000
                    }
                }
            },
        );

        const resp = await alice.sendOutputs([outputData])
        console.log('Transaction is sent', resp)
    } catch (error) {
        console.log('Error: ' + error);
    }
}

async function handleNewOutput(err, data) {
    setTimeout(async () => {
        console.log('Output received:', data)
        const event = JSON.parse(data)
        const outputId = event.event.NewOutput.output.outputId
        console.log(outputId)
        const resp = await bob.collectOutputs([outputId])
        console.log('Output has been collected in the following transaction:', resp)
        process.exit(0)
    }, 15000)
}
run();
