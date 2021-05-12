// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

function run() {
  const { AccountManager } = require('../lib')
  const manager = new AccountManager()
  let address = "atoi1qpheapgshz9saf8mc6zdly96xyz5qdc2qspsv7ezem6fw8lv869ms3amw0g";
  let migration_address = manager.generateMigrationAddress(address);
  console.log(migration_address)
}
run()