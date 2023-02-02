/**
 * This example sends IOTA tokens with an expiration date and claims them.
 */

const getUnlockedManager = require('./account-manager');

let manager
let bob

async function run() {
    try {
        manager = await getUnlockedManager();
        const alice = await manager.getAccount('Alice');
        bob = await manager.getAccount('Bob');

        manager.listen(['NewOutput'], handleNewOutputOfBob)

        const recipientAddress = bob.meta.publicAddresses[0].address
        const amount = '1000000';

        const outputData = await alice.prepareOutput(
            {
                recipientAddress,
                amount,
                unlocks: {
                    expirationUnixTime: Math.round(new Date().getTime() / 1000) + 15000
                }
            },
        );

        const resp = await alice.sendOutputs([outputData])
        console.log('Transaction is sent', resp)

        // Sync account to get the output event
        setTimeout(async () => {
            await bob.sync();
        }, 10000)
    } catch (error) {
        console.log('Error: ', error);
    }
}

async function handleNewOutputOfBob(err, data) {
    try {
        console.log('Output received:', data)
        const event = JSON.parse(data)
        if (event.accountIndex === bob.meta.index) {
            const outputId = event.event.NewOutput.output.outputId
            await bob.sync()
            const resp = await bob.claimOutputs([outputId])
            console.log('Output has been claimed in the following transaction:', resp)
            process.exit(0)
        }
    } catch (error) {
        console.log('Error: ', error);
    }
}
run();
