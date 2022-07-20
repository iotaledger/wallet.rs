const getUnlockedManager = require('./account-manager');

async function run() {
    try {
        const manager = await getUnlockedManager();
        const nodeInfo = await manager.getNodeInfo();
        console.log('Node Info:', nodeInfo);
    } catch (error) {
        console.log('Error: ', error);
    }
    process.exit();
}

run();

// Example output:
// Node Info: {
//     nodeInfo: {
//       name: 'HORNET',
//       version: '2.0.0-alpha.25',
//       status: {
//         isHealthy: true,
//         latestMilestone: [Object],
//         confirmedMilestone: [Object],
//         pruningIndex: 0
//       },
//       supportedProtocolVersions: [ 2 ],
//       protocol: {
//         version: 2,
//         networkName: 'dummy-1',
//         bech32HRP: 'rms',
//         minPoWScore: 1500,
//         rentStructure: [Object],
//         tokenSupply: '1450896407249092'
//       },
//       pendingProtocolParameters: [],
//       baseToken: {
//         name: 'Shimmer',
//         tickerSymbol: 'SMR',
//         unit: 'SMR',
//         subunit: 'glow',
//         decimals: 6,
//         useMetricPrefix: false
//       },
//       metrics: {
//         blocksPerSecond: 1.6,
//         referencedBlocksPerSecond: 1.6,
//         referencedRate: 100
//       },
//       features: []
//     },
//     url: 'https://api.testnet.shimmer.network'
//   }