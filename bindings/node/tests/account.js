// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

async function run() {
  try {
    const { AccountManager, StorageType } = require('../lib')
    const manager = new AccountManager({
      storagePath: './test-database',
      storageType: StorageType.Stronghold
    })
    manager.setStrongholdPassword('password')

    const mnemonic = 'error morning duty ring fiscal uniform erupt woman push march draw flower pair hello cousin real invest region message chief property vital dismiss moment'
    const account = manager.createAccount({
      mnemonic,
      clientOptions: { node: 'http://localhost:14265' }
    })
    account.setAlias('banana')
  } finally {
    const fs = require('fs')
    try {
    fs.rmdirSync('./test-database', { recursive: true })
    } catch (e) {
      // ignore it
    }
  }
}

run()