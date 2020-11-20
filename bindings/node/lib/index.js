var addon = require('../native')
const { AccountManager, Account, SyncedAccount } = addon

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

Account.prototype.sync = promisify(Account.prototype.sync)
SyncedAccount.prototype.send = promisify(SyncedAccount.prototype.send)
SyncedAccount.prototype.retry = promisify(SyncedAccount.prototype.retry)
SyncedAccount.prototype.reattach = promisify(SyncedAccount.prototype.reattach)
SyncedAccount.prototype.promote = promisify(SyncedAccount.prototype.promote)

module.exports = { AccountManager }
