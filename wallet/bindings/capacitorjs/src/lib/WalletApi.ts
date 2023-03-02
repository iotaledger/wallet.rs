// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { AccountId, AccountSyncOptions, CreateAccountPayload } from '../types/account';
import { AccountManagerOptions } from '../types';
import { Account } from './Account';
import { AccountManager } from './AccountManager';

type ProfileManager = Awaited<ReturnType<typeof AccountManager>>
type RecoverAccounts = Parameters<ProfileManager['recoverAccounts']>
interface RecoverAccountsPayload {
    accountStartIndex: number, 
    accountGapLimit: number, 
    addressGapLimit: number, 
    syncOptions?: AccountSyncOptions
}
interface ProfileManagers {
    [key: string]: ProfileManager
}
const profileManagers: ProfileManagers = {}

const WalletApi = {
    async createAccountManager(id: AccountId, options: AccountManagerOptions) {
        const manager = await AccountManager(options)
        manager.id = id
        profileManagers[id] = manager
        return manager
    },
    async createAccount(managerId: AccountId, payload: CreateAccountPayload) {
        const manager = profileManagers[managerId]
        const account = await manager.createAccount(payload)
        bindMethodsAcrossContextBridge(Account.prototype, account)
        return account
    },
    deleteAccountManager(managerId: AccountId) {
        if (managerId && managerId in profileManagers) {
            delete profileManagers[managerId]
        }
    },
    async getAccount(managerId: AccountId, index: number) {
        const manager = profileManagers[managerId]
        const account = await manager.getAccount(index)
        bindMethodsAcrossContextBridge(Account.prototype, account)
        return account
    },
    async getAccounts(managerId: AccountId) {
        const manager = profileManagers[managerId]
        const accounts = await manager.getAccounts()
        accounts.forEach((account) => bindMethodsAcrossContextBridge(Account.prototype, account))
        return accounts
    },
    async recoverAccounts(managerId: AccountId, payload: RecoverAccountsPayload) {
        const manager = profileManagers[managerId]
        const accounts = await manager.recoverAccounts(...Object.values(payload) as RecoverAccounts)
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
