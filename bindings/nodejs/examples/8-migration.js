/**
 * This example creates a new database and account,
 * and migrate funds from the legacy network to the chrysalis network
 */

require('dotenv').config()

const ADDRESS_SECURITY_LEVEL = 2
// Minimum balance that is required for a migration bundle, because of the dust protection in the new network
const MINIMUM_MIGRATION_BALANCE = 1000000
// This value shouldn't be too high, because then the PoW could take to long to get it confirmed
const MAX_INPUTS_PER_BUNDLE = 10


async function run() {
  const { AccountManager, SignerType } = require('@iota/wallet')
  const manager = new AccountManager({
    storagePath: './migration-database',
  })
  manager.setStrongholdPassword(process.env.SH_PASSWORD)
  // Save this mnemonic securely. If you lose it, you potentially lose everything.
  const mnemonic = manager.generateMnemonic()
  console.log("Save this mnemonic securely. If you lose it, you potentially lose everything:", mnemonic);
  manager.storeMnemonic(SignerType.Stronghold, mnemonic)

  const account = await manager.createAccount({
    clientOptions: { node: "https://chrysalis-nodes.iota.cafe", localPow: true, network: "chrysalis-mainnet" },
    alias: 'Migration',
  })

  console.log('Account created:', account.alias())

  const nodes = ['https://nodes.iota.org']
  const seed = process.env.MIGRATION_SEED
  const migrationData = await manager.getMigrationData(
    nodes,
    seed,
    {
      permanode: 'https://chronicle.iota.org/api',
      securityLevel: ADDRESS_SECURITY_LEVEL
    }
  )
  console.log(migrationData)

  let input_batches = getMigrationBundles(migrationData.inputs)

  for (inputs of input_batches) {
    const bundle = await manager.createMigrationBundle(seed, migrationData.inputs.map(input => input.index), {
      logFileName: 'iota-migration.log',
      // if the input is a spent address we do a bundle mining process which takes 10 minutes to reduce the amount 
      // of the parts of the private key which get revealed
      mine: inputs.inputs[0].spent
    })
    await manager.sendMigrationBundle(nodes, bundle.bundleHash)
  }
}

run()

const getMigrationBundles = (inputs) => {
  // Categorise spent vs unspent inputs
  const { spent, unspent } = inputs.reduce((acc, input) => {
    if (input.spent) {
      acc.spent.push(input)
    } else {
      acc.unspent.push(input)
    }
    return acc;
  }, { spent: [], unspent: [] })
  const unspentInputChunks = selectInputsForUnspentAddresses(unspent)
  const spentInputs = spent.filter((input) => input.balance >= MINIMUM_MIGRATION_BALANCE)
  return [
    ...spentInputs.map((input) => ({
      // Make sure for spent addresses, we only have one input per bundle    
      inputs: [input]
    })),
    ...unspentInputChunks.map((inputs) => ({ inputs }))
  ]
};
/**
 * Prepares inputs (as bundles) for unspent addresses.
 * Steps:
 *   - Categorises inputs in two groups 1) inputs with balance >= MINIMUM_MIGRATION_BALANCE 2) inputs with balance < MINIMUM_MIGRATION_BALANCE
 *   - Creates chunks of category 1 input addresses such that length of each chunk should not exceed MAX_INPUTS_PER_BUNDLE
 *   - For category 2: 
 *         - Sort the inputs in descending order based on balance;
 *         - Pick first N inputs (where N = MAX_INPUTS_PER_BUNDLE) and see if their accumulative balance >= MINIMUM_MIGRATION_BALANCE
 *         - If yes, then repeat the process for next N inputs. Otherwise, iterate on the remaining inputs and add it to a chunk that has space for more inputs
 *         - If there's no chunk with space left, then ignore these funds. NOTE THAT THESE FUNDS WILL ESSENTIALLY BE LOST!
 * 
 * NOTE: If the total sum of provided inputs are less than MINIMUM_MIGRATION_BALANCE, then this method will just return and empty array as those funds can't be migrated.
 * 
 * This method gives precedence to max inputs over funds. It ensures, a maximum a bundle could have is 30 inputs and their accumulative balance >= MINIMUM_MIGRATION_BALANCE
 * 
 * @method selectInputsForUnspentAddresses
 * 
 * @params {Input[]} inputs
 * 
 * @returns {Input[][]}
 */
const selectInputsForUnspentAddresses = (inputs) => {
  const totalInputsBalance = inputs.reduce((acc, input) => acc + input.balance, 0);

  // If the total sum of unspent addresses is less than MINIMUM MIGRATION BALANCE, just return an empty array as these funds cannot be migrated
  if (totalInputsBalance < MINIMUM_MIGRATION_BALANCE) {
    return [];
  }

  const { inputsWithEnoughBalance, inputsWithLowBalance } = inputs.reduce((acc, input) => {
    if (input.balance >= MINIMUM_MIGRATION_BALANCE) {
      acc.inputsWithEnoughBalance.push(input);
    } else {
      acc.inputsWithLowBalance.push(input);
    }

    return acc;
  }, { inputsWithEnoughBalance: [], inputsWithLowBalance: [] })

  let chunks = inputsWithEnoughBalance.reduce((acc, input, index) => {
    const chunkIndex = Math.floor(index / MAX_INPUTS_PER_BUNDLE)

    if (!acc[chunkIndex]) {
      acc[chunkIndex] = [] // start a new chunk
    }

    acc[chunkIndex].push(input)

    return acc
  }, [])

  const fill = (_inputs) => {
    _inputs.every((input) => {
      const chunkIndexWithSpaceForInput = chunks.findIndex((chunk) => chunk.length < MAX_INPUTS_PER_BUNDLE);

      if (chunkIndexWithSpaceForInput > -1) {
        chunks = chunks.map((chunk, idx) => {
          if (idx === chunkIndexWithSpaceForInput) {
            return [...chunk, input]
          }

          return chunk
        })

        return true;
      }

      // If there is no space, then exit
      return false;
    })
  }

  const totalBalanceOnInputsWithLowBalance = inputsWithLowBalance.reduce((acc, input) => acc + input.balance, 0)

  // If all the remaining input addresses have accumulative balance less than the minimum migration balance,
  // Then sort the inputs in descending order and try to pair the
  if (totalBalanceOnInputsWithLowBalance < MINIMUM_MIGRATION_BALANCE) {
    const sorted = inputsWithLowBalance.slice().sort((a, b) => b.balance - a.balance)

    fill(sorted)
  } else {
    let startIndex = 0

    const sorted = inputsWithLowBalance.slice().sort((a, b) => b.balance - a.balance)
    const max = Math.ceil(sorted.length / MAX_INPUTS_PER_BUNDLE);

    while (startIndex < max) {
      const inputsSubset = sorted.slice(startIndex * MAX_INPUTS_PER_BUNDLE, (startIndex + 1) * MAX_INPUTS_PER_BUNDLE)
      const balanceOnInputsSubset = inputsSubset.reduce((acc, input) => acc + input.balance, 0);

      if (balanceOnInputsSubset >= MINIMUM_MIGRATION_BALANCE) {
        chunks = [...chunks, inputsSubset]
      } else {
        fill(inputsSubset)
      }

      startIndex++;
    }
  }

  return chunks;
};