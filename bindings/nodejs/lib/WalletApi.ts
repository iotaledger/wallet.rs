// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { AccountId, CreateAccountPayload } from '../types/account'
import { AccountManagerOptions } from '../types'
import { AccountManager } from './AccountManager'

type ProfileManager = Awaited<ReturnType<typeof AccountManager>>
type RecoverAccounts = NonNullable<Parameters<ProfileManager['recoverAccounts']>>
interface ProfileManagers {
    [key: string]: ProfileManager
}
const profileManagers: ProfileManagers = {}
// @ts-ignore
export function WalletApi(): any {
    return {
        async createAccountManager(id: AccountId, options: AccountManagerOptions) {
            const manager = await AccountManager(options)
            manager.id = id
            profileManagers[id] = manager
            return manager
        },
        async createAccount(managerId: AccountId, payload: CreateAccountPayload) {
            const manager = profileManagers[managerId]
            const account = await manager.createAccount(payload)
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
            return account
        },
        async getAccounts(managerId: AccountId) {
            const manager = profileManagers[managerId]
            const accounts = await manager.getAccounts()
            return accounts
        },
        async recoverAccounts(managerId: AccountId, payload: RecoverAccounts) {
            const manager = profileManagers[managerId]
            const accounts = await manager.recoverAccounts(...Object.values(payload) as RecoverAccounts)
            return accounts
        },
    }
}

