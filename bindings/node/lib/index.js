var addon = require('../native')
const { AccountManager, Account } = addon

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

module.exports = { AccountManager }
