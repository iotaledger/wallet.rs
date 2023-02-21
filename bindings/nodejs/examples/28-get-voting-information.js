/**
 * This example creates a new voting event
 */
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0')

        const node = { url: process.env.NODE_URL }
        const eventIds = await account.getParticipationEventIds(node);
        console.log('Event IDs from the node:', eventIds)

        // store the event information from a node locally
        const registeredEvents = await account.registerParticipationEvents({
            node: { url: process.env.NODE_URL },
            eventsToIgnore: [eventIds[0]],
        })
        const eventId = Object.keys(registeredEvents)[0]

        // get the participation events that are stored in wallet.rs
        const events = await account.getParticipationEvents()
        console.log('List of registered events:', events)

        // get the participation event with a specific id that is stored in wallet.rs
        const event = await account.getParticipationEvent(eventId)
        console.log(`Event ${eventId}:`, event)

        // get the status of a participation event that is stored in wallet.rs
        const eventStatus = await account.getParticipationEventStatus(eventId)
        console.log('Event Status:', eventStatus)

        // removes the participation events from a manager
        await account.deregisterParticipationEvent(eventId)

        process.exit(0)
    } catch (error) {
        console.log('Error: ', error);
        process.exit(1);
    }
}

run();
