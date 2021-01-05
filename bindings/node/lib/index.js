// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

var addon = require('../native')
const { AccountManager, Account, SyncedAccount, EventListener, initLogger } = addon

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
AccountManager.prototype.syncAccounts = promisify(AccountManager.prototype.syncAccounts)
AccountManager.prototype.internalTransfer = promisify(AccountManager.prototype.internalTransfer)

module.exports = {
  AccountManager,
  addEventListener,
  initLogger: config => initLogger(JSON.stringify(config)),
  RemainderValueStrategy,
  SignerType: {
    Stronghold: 1,
    EnvMnemonic: 2
  }
}
