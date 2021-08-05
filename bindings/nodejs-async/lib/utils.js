// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

function promisify(fn) {
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

class RemainderValueStrategy {
    changeAddress() {
        return "ChangeAddress";
    }
    reuseAddress() {
        return "ReuseAddress";
    }
    accountAddress(address) {
        return address;
    }
}

module.exports.promisify = promisify;
module.exports.RemainderValueStrategy = new RemainderValueStrategy();