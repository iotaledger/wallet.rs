require('dotenv').config();

/**
 * This example creates a new database and account
 * Please make sure you have the .env variables configured
 */

async function run() {
    const { AccountManager, StorageType, SignerType } = require('iota-wallet')
    const manager = new AccountManager({
        storagePath: './test-database',
        storageType: StorageType.Sqlite
    })
    
    const mnemonic = process.env.IOTA_WALLET_MNEMONIC
    
    const account = await manager.createAccount({
      mnemonic,
      alias: 'Test',
      clientOptions: { node: process.env.NODE_URL, localPow: false },
      signerType: SignerType.EnvMnemonic
    })

    console.log('alias', account.alias())
    console.log('balance', account.balance())
    
    console.log('syncing account now')

    const synced = await account.sync()

    console.log('synced', synced)
    console.log('acc messages', account.listMessages())
    console.log('acc spent addresses', account.listAddresses(false))
    console.log('acc unspent addresses', account.listAddresses(true))
}

run()