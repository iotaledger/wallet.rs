/**
 * This example creates a new database and account
 * Please make sure you have the .env variables configured
 */

async function run() {
    const { AccountManager, SignerType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './alice-database',
    })
    manager.setStrongholdPassword("your_password")
    manager.storeMnemonic(SignerType.Stronghold, manager.generateMnemonic())

    const account = await manager.createAccount({
        clientOptions: { node: "https://api.lb-0.testnet.chrysalis2.com", localPow: true },
        alias: 'Alice',
    })

    console.log('Account created:', account.alias())
}

run()
