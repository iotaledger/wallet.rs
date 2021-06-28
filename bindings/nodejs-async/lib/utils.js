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

module.exports.promisify = promisify;