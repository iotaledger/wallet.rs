/**
 * This example creates a new voting event
 */
require('dotenv').config({ path: '../.env' });
const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();

        const account = await manager.getAccount('0');

        const eventIds = await account.getParticipationEventIds();
        console.log('Event ids from the node:', eventIds)
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
