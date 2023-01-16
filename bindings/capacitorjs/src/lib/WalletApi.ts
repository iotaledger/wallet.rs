// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Account } from './Account';
import { AccountManager } from './AccountManager';

const profileManagers = {}
const WalletApi = {
    async createAccountManager(
        id: number, // Account.AccountId,
        options: unknown,  // Account.AccountManagerOptions
    ) {
        console.error({options})
        const manager = await AccountManager(options)
        manager.id = id
        console.error({manager})
        profileManagers[id] = manager
        // bindMethodsAcrossContextBridge(WalletApi.AccountManager, manager)
        return manager
    },
    async createAccount(managerId, payload) {
        const manager = profileManagers[managerId]
        const account = await manager.createAccount(payload)
        bindMethodsAcrossContextBridge(Account.prototype, account)
        return account
    },
    deleteAccountManager(id) {
        if (id && id in profileManagers) {
            delete profileManagers[id]
        }
    },
    async getAccount(managerId, index) {
        const manager = profileManagers[managerId]
        console.error({manager})
        const account = await manager.getAccount(index)
        bindMethodsAcrossContextBridge(Account.prototype, account)
        return account
    },
    async getAccounts(managerId) {
        const manager = profileManagers[managerId]
        console.error({manager})
        const accounts = await manager.getAccounts()
        accounts.forEach((account) => bindMethodsAcrossContextBridge(Account.prototype, account))
        return accounts
    },
    async recoverAccounts(managerId, payload) {
        const manager = profileManagers[managerId]
        console.error({manager})
        const accounts = await manager.recoverAccounts(...Object.values(payload))
        accounts.forEach((account) => bindMethodsAcrossContextBridge(Account.prototype, account))
        return accounts
    },
}
function bindMethodsAcrossContextBridge(prototype, object) {
    const prototypeProperties = Object.getOwnPropertyNames(prototype)
    prototypeProperties.forEach((key) => {
        if (key !== 'constructor') {
            object[key] = object[key].bind(object)
        }
    })
}

export { WalletApi };
