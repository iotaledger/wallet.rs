// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../native')
let { AccountManager, Account, SyncedAccount, EventListener, initLogger } = addon

function promisify (fn) {
  return function () {
    return new Promise((resolve, reject) => fn.apply(this, [...Array.from(arguments), (err, data) => {
      if (err) {
        reject(err)
      } else {
        resolve(data)
      }
    }]))
  }
}

class EventListenerWrapper {
  constructor(event, cb) {
    const instance = new EventListener(event)
    this.poll(instance, cb)
  }

  poll(instance, cb) {
    instance.poll((err, data) => {
      cb(err, err ? null : JSON.parse(data))
      this.poll(instance, cb)
    })
  }
}

function addEventListener (event, cb) {
  new EventListenerWrapper(event, cb)
}

class RemainderValueStrategy {
  constructor(strategyName, payload) {
    this.strategy = strategyName
    this.value = payload || null
  }

  static changeAddress() {
    return new RemainderValueStrategy('ChangeAddress')
  }

  static reuseAddress() {
    return new RemainderValueStrategy('ReuseAddress')
  }

  static accountAddress(address) {
    return new RemainderValueStrategy('AccountAddress', address)
  }
}

Account.prototype.sync = promisify(Account.prototype.sync)
const send = SyncedAccount.prototype.send
SyncedAccount.prototype.send = function (address, amount, options) {
  if (options && (typeof options === 'object') && options.indexation && options.indexation.data) {
    return promisify(send).apply(this, [address, amount, {
      remainderValueStrategy: options.remainderValueStrategy,
      indexation: {
        index: options.indexation.index,
        data: Array.from(options.indexation.data),
      }
    }])
  } else {
    return promisify(send).apply(this, options ? [address, amount, options] : [address, amount])
  }
}
SyncedAccount.prototype.retry = promisify(SyncedAccount.prototype.retry)
SyncedAccount.prototype.reattach = promisify(SyncedAccount.prototype.reattach)
SyncedAccount.prototype.promote = promisify(SyncedAccount.prototype.promote)

const managerClass = AccountManager
AccountManager = function () {
  const instance = new managerClass(arguments[0])

  // workaround to force the manager to cleanup
  // this is needed because somehow the manager `drop` impl isn't being called - issue on Neon
  const cleanup = () => {
    try {
      instance.stopBackgroundSync()
    }
    finally {
      process.exit()
    }
  }

  process.on('exit', cleanup)
  process.on('SIGINT', cleanup)
  process.on('SIGTERM', cleanup)
  process.on('SIGHUP', cleanup)
  process.on('SIGBREAK', cleanup)

  return instance
}
AccountManager.prototype = managerClass.prototype
AccountManager.prototype.syncAccounts = promisify(AccountManager.prototype.syncAccounts)
AccountManager.prototype.internalTransfer = promisify(AccountManager.prototype.internalTransfer)

module.exports = {
  AccountManager,
  addEventListener,
  initLogger: config => initLogger(JSON.stringify(config)),
  RemainderValueStrategy,
  SignerType: {
    Stronghold: 1
  },
  StorageType: {
    Sqlite: { type: 'Sqlite' },
    Stronghold: { type: 'Stronghold' }
  },
  MessageType: {
    Received: 1,
    Sent: 2,
    Failed: 3,
    Unconfirmed: 4,
    Value: 5
  }
}
