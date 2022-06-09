/**
 * This example changes the stronghold password.
 */
 require('dotenv').config();
const getUnlockedManager = require('./account-manager');

const NEW_PASSWORD = 'new_super_secure_password'

async function run() {
    try {
        const manager = await getUnlockedManager();

        await manager.changeStrongholdPassword(NEW_PASSWORD)
        await sendAmountIfManagerUnlocked(manager)

        await manager.clearStrongholdPassword()
        await shouldNotSendAmount(manager)
    
        await manager.setStrongholdPassword(NEW_PASSWORD)
        await sendAmountIfManagerUnlocked(manager)

        // Reverts to original password
        await manager.changeStrongholdPassword(process.env.SH_PASSWORD)
    } catch (error) {
        console.log('Error: ' + error);
    }
    process.exit(0);
}

async function sendAmountIfManagerUnlocked(manager) {
    const account = await manager.getAccount('0')
    const response = await account.sendAmount([
        {
            address: 'rms1qrrv7flg6lz5cssvzv2lsdt8c673khad060l4quev6q09tkm9mgtupgf0h0',
            amount: '300000',
        },
    ]);
    console.log('Transaction sent', response)
}


async function shouldNotSendAmount(manager) {
    try {
        await sendAmountIfManagerUnlocked(manager)
    } catch {
        console.log('Account manager locked as expected!')
        return
    }
    throw new Error('Account manager should not be accessible')
}

run();
