// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const { AccountManager } = require('@iota/wallet');
require('dotenv').config({ path: '.env' });

const v2Path = '../../../../tests/fixtures/v2_with_backup.stronghold';
const v3Path = './migration-database/wallet.stronghold';

// Run with command:
// node run-example wallet/migrate-stronghold-snapshot-v2-to-v3.js

async function run() {
    const manager = new AccountManager({
        storagePath: './migration-database',
    });

    manager.migrateStrongholdSnapshotV2ToV3(
        v2Path,
        'current_password',
        'wallet.rs',
        100,
        v3Path,
        // Optional, can also stay the same password
        'new_password',
    );
    console.log('Migrated stronghold');

    // This shouldn't fail anymore as snapshot has been migrated.
    manager.setStrongholdPassword('new_password');
}

run().then(() => process.exit());
