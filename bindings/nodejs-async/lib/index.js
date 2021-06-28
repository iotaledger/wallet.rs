// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../index.node');
const mh = require("./messageHandler.js");
const el = require("./eventListener.js");
console.log(mh);
let { initLogger } = addon;
let { MessageHandler } = mh;
let { EventListener } = el;

// function promisify(fn) {
//   return function () {
//     return new Promise((resolve, reject) => fn.apply(this, [...Array.from(arguments), (err, data) => {
//       if (err) {
//         reject(err)
//       } else {
//         resolve(data)
//       }
//     }]))
//   }
// }
// console.log(process.versions);
// try {
//   var k = new Map();
//   console.log("ES6 supported!!")
// } catch(err) {
//   console.log("ES6 not supported :(")
// }

// try {
//   var k = new HashMap();
//   console.log("ES100 supported!!")
// } catch(err) {
//   console.log("ES100 not supported :(")
// }


// console.log(addon)


// const sendMessageAsync = promisify(sendMessage);
// const _id = "1";
// class MessageHandler {
//   constructor(options) {
//     console.log("MessageHandler constructor called.");
//     this.messageHandler = messageHandlerNew(JSON.stringify(options));
//   }

//   async sendMessage(message) {
//     console.log("sendMessage called.");
//     console.log(message);
//     return sendMessageAsync(JSON.stringify(message), this.messageHandler);
//   }

//   async getAccount(accountId) {
//     return this.sendMessage({
//       id: _id,
//       cmd: "GetAccount",
//       payload: accountId,
//     });
//   }

//   async getAccounts() {
//     return this.sendMessage({
//       id: _id,
//       cmd: "GetAccounts",
//     });
//   }

//   async createAccount(account) {
//     return this.sendMessage({
//       id: _id,
//       cmd: "CreateAccount",
//       payload: account
//     });
//   }

//   async setStrongholdPassword(password) {
//     return this.sendMessage({
//       id: _id,
//       cmd: "SetStrongholdPassword",
//       payload: password,
//     });
//   }

//   async storeMnemonic(mnemonic) {
//     return this.sendMessage({
//       id: _id,
//       cmd: "StoreMnemonic",
//       payload: {
//         signerType: {
//           type: 'Stronghold'
//         },
//         mnemonic
//       },
//     });
//   }

// };




// enum AccountMethod {
//   GenerateAddress,
//   GetUnusedAddress,
//   ListMessages,
//   ListAddresses,
//   GetBalance,
//   GetLatestAddress,
//   SyncAccount,
//   IsLatestAddressUnused,
//   SetAlias,
//   GetNodeInfo,
// }

initLogger(JSON.stringify({
  color_enabled: true,
  outputs: [{
    name: 'wallet.log',
    level_filter: 'debug'
  }]
}));

// const messageHandler = new MessageHandler();




module.exports = {
  MessageHandler,
  EventListener,
  initLogger
};
// test();
// export declare interface AccountToCreate {
//   clientOptions: ClientOptions;
//   alias?: string;
//   createdAt?: string;
//   signerType?: SignerType;
//   skipPersistence?: boolean;
// }




// async function startBackgroundSync(pollingInterval: number, automaticOutputConsolidation: boolean): Promise<void>
// async function stopBackgroundSync(): void
// async function setStoragePassword(password: string): void

// async function changeStrongholdPassword(currentPassword: string, newPassword: string): void
// async function generateMnemonic(): string
// async function storeMnemonic(signerType: SignerType, mnemonic?: string): void
// async function createAccount(account: AccountToCreate): Account
// async function getAccount(accountId) {
//     return messageHandler.sendMessage({
//       id: accountId,
//       cmd: "GetAccounts",
//       payload: accountId,
//     });
// }
// async function getAccounts(): Account[]
// async function removeAccount(accountId: string | number): void
// async function syncAccounts(options?: SyncOptions): Promise<SyncedAccount[]>
// async function internalTransfer(fromAccount: Account, toAccount: Account, amount: number): Promise<Message>
// async function backup(destination: string, password: string): string
// async function importAccounts(source: string, password: string): void
// async function isLatestAddressUnused(): Promise<boolean>
// async function setClientOptions(options: ClientOptions): void
// migration
// async function getMigrationData(nodes: string[], seed: string, options?: MigrationDataOptions): Promise<MigrationData>
// async function createMigrationBundle(seed: string, inputAddressIndexes: number[], options?: CreateMigrationBundleOptions): Promise<MigrationBundle>
// async function sendMigrationBundle(nodes: string[], bundleHash: string, options?: SendMigrationBundleOptions): Promise<void>
// events
// async function getBalanceChangeEvents(count?: number, skip?: number, fromTimestamp?: number): BalanceChangeEvent[]
// async function getBalanceChangeEventCount(fromTimestamp?: number): number
// async function getTransactionConfirmationEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionConfirmationEvent[]
// async function getTransactionConfirmationEventCount(fromTimestamp?: number): number
// async function getNewTransactionEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
// async function getNewTransactionEventCount(fromTimestamp?: number): number
// async function getReattachmentEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
// async function getReattachmentEventCount(fromTimestamp?: number): number
// async function getBroadcastEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
// async function getBroadcastEventCount(fromTimestamp?: number): number







// class EventListenerWrapper {
//   constructor(event, cb) {
//     const instance = new EventListener(event)
//     this.poll(instance, cb)
//   }

//   poll(instance, cb) {
//     instance.poll((err, data) => {
//       cb(err, err ? null : JSON.parse(data))
//       this.poll(instance, cb)
//     })
//   }
// }

// function addEventListener (event, cb) {
//   new EventListenerWrapper(event, cb)
// }

// class RemainderValueStrategy {
//   constructor(strategyName, payload) {
//     this.strategy = strategyName
//     this.value = payload || null
//   }

//   static changeAddress() {
//     return new RemainderValueStrategy('ChangeAddress')
//   }

//   static reuseAddress() {
//     return new RemainderValueStrategy('ReuseAddress')
//   }

//   static accountAddress(address) {
//     return new RemainderValueStrategy('AccountAddress', address)
//   }
// }

// Account.prototype.sync = promisify(Account.prototype.sync)
// Account.prototype.isLatestAddressUnused = promisify(Account.prototype.isLatestAddressUnused)

// const send = Account.prototype.send
// Account.prototype.send = function (address, amount, options) {
//   if (options && (typeof options === 'object')) {
//     const formattedOptions = options
//     if (options.indexation) {
//       let index = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.index) :  options.indexation.index
//       let data = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.data) :  options.indexation.data
//       formattedOptions.indexation = {
//         index: Array.from(index),
//         data: data ? Array.from(data) : null,
//       }
//     }
//     if (options.remainderValueStrategy) {
//       formattedOptions.remainderValueStrategy = options.remainderValueStrategy
//     }
//     return promisify(send).apply(this, [address, amount, formattedOptions])
//   } else {
//     return promisify(send).apply(this, options ? [address, amount, options] : [address, amount])
//   }
// }

// const sendToMany = Account.prototype.sendToMany
// Account.prototype.sendToMany = function (outputs, options) {
//   if (options && (typeof options === 'object')) {
//     const formattedOptions = options
//     if (options.indexation) {
//       let index = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.index) :  options.indexation.index
//       let data = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.data) :  options.indexation.data
//       formattedOptions.indexation = {
//         index: Array.from(index),
//         data: data ? Array.from(data) : null,
//       }
//     }
//     if (options.remainderValueStrategy) {
//       formattedOptions.remainderValueStrategy = options.remainderValueStrategy
//     }
//     return promisify(sendToMany).apply(this, [outputs, formattedOptions])
//   } else {
//     return promisify(sendToMany).apply(this, options ? [outputs, options] : [outputs])
//   }
// }


// Account.prototype.retry = promisify(Account.prototype.retry)
// Account.prototype.reattach = promisify(Account.prototype.reattach)
// Account.prototype.promote = promisify(Account.prototype.promote)
// Account.prototype.consolidateOutputs = promisify(Account.prototype.consolidateOutputs)
// Account.prototype.getNodeInfo = promisify(Account.prototype.getNodeInfo)

// AccountManager.prototype.syncAccounts = promisify(AccountManager.prototype.syncAccounts)
// AccountManager.prototype.internalTransfer = promisify(AccountManager.prototype.internalTransfer)
// AccountManager.prototype.isLatestAddressUnused = promisify(AccountManager.prototype.isLatestAddressUnused)
// AccountManager.prototype.getMigrationData = promisify(AccountManager.prototype.getMigrationData)
// AccountManager.prototype.createMigrationBundle = promisify(AccountManager.prototype.createMigrationBundle)
// AccountManager.prototype.sendMigrationBundle = promisify(AccountManager.prototype.sendMigrationBundle)

// module.exports = {
//   AccountManager,
//   addEventListener,
//   initLogger: config => initLogger(JSON.stringify(config)),
//   RemainderValueStrategy,
//   SignerType: {
//     Stronghold: 1
//   },
//   MessageType: {
//     Received: 1,
//     Sent: 2,
//     Failed: 3,
//     Unconfirmed: 4,
//     Value: 5,
//     Confirmed: 6
//   }
// }
