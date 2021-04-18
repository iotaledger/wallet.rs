/**
 * This example creates a new database and account,
 * and migrate funds from the legacy network to the chrysalis network
 */

require('dotenv').config()

async function run() {
    const { AccountManager, SignerType } = require('@iota/wallet')
    const manager = new AccountManager({
        storagePath: './migration-database',
    })
    manager.setStrongholdPassword(process.env.SH_PASSWORD)
    manager.storeMnemonic(SignerType.Stronghold)

    const account = await manager.createAccount({
        clientOptions: { node: "https://api.lb-0.testnet.chrysalis2.com", localPow: true },
        alias: 'Migration',
    })

    console.log('Account created:', account.alias())

    const nodes = ['https://nodes.devnet.iota.org']
    const seed = process.env.MIGRATION_SEED
    const migrationData = await manager.getMigrationData(
      nodes,
      seed,
      {
        permanode: 'https://chronicle.iota.org/api'
      }
    )
    console.log(migrationData)
    const bundle = await manager.createMigrationBundle(seed, migrationData.inputs.map(input => input.index), {
      logFileName: 'iota-migration.log'
    })
    await manager.sendMigrationBundle(nodes, bundle.bundleHash)
}

run()
