/**
 * This example shows how to work with the participation plugin.
 */
const getUnlockedManager = require('./account-manager');

// TODO: replace this with own event
const EVENT_ID = '0x7ba318a26a1f639389a3428f159f40aebbcc776a4f8ca17de4fa45221ac79fbd'

async function waitAndSync(account) {
    return new Promise((resolve) => {
        setTimeout(async () => {
            await account.sync();
            resolve()
        }, 10000)
    })
}


async function run() {
    let transaction
    let participationOverview
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');
        await account.sync()

        // Start with increasing your voting power to vote
        transaction = await account.increaseVotingPower('5000000')
        console.log('Increase Voting Power Transaction:', transaction)
        await waitAndSync(account)

        // Check your available voting power
        const votingPower = await account.getVotingPower()
        console.log('Voting Power:', votingPower);

        // Once the transaction went through, call the vote method 
        transaction = await account.vote(EVENT_ID, [0])
        console.log('Voting Transaction:', transaction)
        await waitAndSync(account)
        
        // Check the votes you have participated in
        participationOverview = await account.getParticipationOverview();
        console.log('Participation Overview:', JSON.stringify(participationOverview));

        // Decrease your voting power
        transaction = await account.decreaseVotingPower('500000')
        console.log('Decrease Voting Power Transaction', transaction)
        await waitAndSync(account)

        // Check the votes you have participated in
        participationOverview = await account.getParticipationOverview();
        console.log('Participation Overview:', JSON.stringify(participationOverview));
        
        // Stop voting for a given event
        transaction = await account.stopParticipating(EVENT_ID);
        console.log('Stop Participation Transaction', transaction)
        await waitAndSync(account)

    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit(0);
}

run();
 