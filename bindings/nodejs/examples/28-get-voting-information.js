/**
 * This example creates a new voting event
 */
const getUnlockedManager = require('./account-manager');

console.log(process.env)

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0')

        const node = { url: 'https://api.testnet.shimmer.network' }//process.env.NODE_URL }
        const eventIds = await account.getParticipationEventIds(node);
        console.log('Event IDs from the node:', eventIds)
        let EVENT_ID = eventIds[0]

        // store the event information from a node locally
        await account.registerParticipationEvent(
            EVENT_ID,
            [{ url: process.env.NODE_URL }]
        )

        // get the participation events that are stored in wallet.rs
        const events = await account.getParticipationEvents()
        console.log('List of registered events:', events)

        // get the participation event with a specific id that is stored in wallet.rs
        const event = await account.getParticipationEvent(EVENT_ID)
        console.log(`Event ${EVENT_ID}:`, event)

        // get the status of a participation event that is stored in wallet.rs
        const eventStatus = await account.getParticipationEventStatus(EVENT_ID)
        console.log('Event Status:', eventStatus)

        // removes the participation events from a manager
        await account.deregisterParticipationEvent(EVENT_ID)

        process.exit(0)
    } catch (error) {
        console.log('Error: ', error);
        process.exit(1);
    }
}

run();
