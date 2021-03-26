// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const addon = require('../build/Release')
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
Account.prototype.isLatestAddressUnused = promisify(Account.prototype.isLatestAddressUnused)

function rewriteSend(classT) {
  const send = classT.prototype.send
  classT.prototype.send = function (address, amount, options) {
    if (options && (typeof options === 'object') && options.indexation) {
      let index = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.index) :  options.indexation.index
      let data = typeof options.indexation.index === 'string' ? new TextEncoder().encode(options.indexation.data) :  options.indexation.data
      const formattedOptions = {
        indexation: {
          index: Array.from(index),
          data: data ? Array.from(data) : null,
        }
      }
      if (options.remainderValueStrategy) {
        formattedOptions.remainderValueStrategy = options.remainderValueStrategy
      }
      return promisify(send).apply(this, [address, amount, formattedOptions])
    } else {
      return promisify(send).apply(this, options ? [address, amount, options] : [address, amount])
    }
}
}

rewriteSend(Account)
rewriteSend(SyncedAccount)

Account.prototype.retry = promisify(Account.prototype.retry)
Account.prototype.reattach = promisify(Account.prototype.reattach)
Account.prototype.promote = promisify(Account.prototype.promote)
Account.prototype.consolidateOutputs = promisify(Account.prototype.consolidateOutputs)

AccountManager.prototype.syncAccounts = promisify(AccountManager.prototype.syncAccounts)
AccountManager.prototype.internalTransfer = promisify(AccountManager.prototype.internalTransfer)
AccountManager.prototype.isLatestAddressUnused = promisify(AccountManager.prototype.isLatestAddressUnused)

module.exports = {
  AccountManager,
  addEventListener,
  initLogger: config => initLogger(JSON.stringify(config)),
  RemainderValueStrategy,
  SignerType: {
    Stronghold: 1
  },
  MessageType: {
    Received: 1,
    Sent: 2,
    Failed: 3,
    Unconfirmed: 4,
    Value: 5,
    Confirmed: 6
  }
}
