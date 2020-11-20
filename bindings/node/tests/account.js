async function run() {
  try {
    const { AccountManager } = require('../lib')
    const manager = new AccountManager('./test-database')
    manager.setStrongholdPassword('password')

    const mnemonic = 'error morning duty ring fiscal uniform erupt woman push march draw flower pair hello cousin real invest region message chief property vital dismiss moment'
    const account = manager.createAccount({
      mnemonic,
      clientOptions: { node: 'http://localhost:14265' }
    })
    console.log('balance', account.availableBalance())
    const synced = await account.sync({})
    console.log('synced', synced)
    console.log('acc messages', account.listMessages())
    console.log('acc spent addresses', account.listAddresses(false))
    console.log('acc unspent addresses', account.listAddresses(true))
  } finally {
    const fs = require('fs')
    try {
    fs.rmdirSync('./test-database', { recursive: true })
    } catch (e) {
      // ignore it
    }
  }
}

run()