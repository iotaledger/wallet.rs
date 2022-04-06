// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0


import MessageHandler from './messageHandler';
import AccountForMessages from './account';
import EventListener from './eventListener';

import type { IAccountManagerOptions } from './types'

export default class AccountManagerForMessages {
    messageHandler: MessageHandler;
    eventListener: EventListener | null;

    constructor(options: IAccountManagerOptions) {
        this.messageHandler = new MessageHandler(options);
        this.eventListener = null;
    }

    async getAccount(accountId: string) {
        return this.messageHandler
            .sendMessage({
                cmd: 'GetAccount',
                payload: accountId,
            })
            .then(
                (acc) =>
                    new AccountForMessages(
                        JSON.parse(acc).payload,
                        this.messageHandler,
                    ),
            );
    }

    async getAccounts() {
        return this.messageHandler.sendMessage({
            cmd: 'GetAccounts',
        });
    }

    async createAccount(account: any) {
        return this.messageHandler
            .sendMessage({
                cmd: 'CreateAccount',
                payload: account,
            })
            .then(
                (acc) =>
                    new AccountForMessages(
                        JSON.parse(acc).payload,
                        this.messageHandler,
                    ),
            );
    }

    async setStrongholdPassword(password: string) {
        return this.messageHandler.sendMessage({
            cmd: 'SetStrongholdPassword',
            payload: password,
        });
    }

    async storeMnemonic(mnemonic: string) {
        return this.messageHandler.sendMessage({
            cmd: 'StoreMnemonic',
            payload: {
                signerType: {
                    type: 'Stronghold',
                },
                mnemonic,
            },
        });
    }

    async backup(destination: string, password: string) {
        return this.messageHandler.sendMessage({
            cmd: 'Backup',
            payload: {
                destination,
                password,
            },
        });
    }

    async importAccounts(backupPath: string, password: string) {
        return this.messageHandler.sendMessage({
            cmd: 'RestoreBackup',
            payload: {
                backupPath,
                password,
            },
        });
    }

    listen(eventName: string, callback: any) {
        if (this.eventListener == null) {
            this.eventListener = new EventListener({});
        }
        return this.eventListener.listen(eventName, callback);
    }

    removeEventListeners(eventName: string) {
        if (this.eventListener == null) {
            return;
        }

        if (
            this.eventListener.removeEventListeners(
                eventName,
            ) <= 0
        ) {
            this.eventListener = null;
            return;
        }
    }
}
