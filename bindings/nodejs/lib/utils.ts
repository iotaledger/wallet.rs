// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// @ts-nocheck
export function promisify(fn) {
  return function () {
    return new Promise((resolve, reject) =>
      fn.apply(this, [
        ...Array.from(arguments),
        (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve(data);
          }
        },
      ]),
    );
  };
}

class RemainderValueStrategy {
  changeAddress() {
    return {
      strategy: 'ChangeAddress',
      value: null,
    };
  }

  reuseAddress() {
    return {
      strategy: 'ReuseAddress',
      value: null,
    };
  }

  accountAddress(address) {
    return {
      strategy: 'AccountAddress',
      value: address,
    };
  }
}

export class OutputKind {
  constructor() {}

  static signatureLockedSingle() {
    return 'SignatureLockedSingle';
  }

  static signatureLockedDustAllowance() {
    return 'SignatureLockedDustAllowance';
  }
}

export const remainderValueStrategy =  new RemainderValueStrategy();



