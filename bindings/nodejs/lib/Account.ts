// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
import type {
    AccountBalance,
    Address,
    AccountSyncOptions,
    AccountMeta,
    NodeInfo,
    ClientOptions,
    OutputsToCollect,
    OutputData,
    Transaction,
    NativeTokenOptions,
    TransferOptions,
    NftOptions,
    AddressWithAmount,
    AddressMicroAmount,
    AddressNativeTokens,
    AddressNftId
} from '../types';

export class Account {
    meta: AccountMeta;
    private messageHandler: MessageHandler

    constructor(
        accountMeta: AccountMeta,
        messageHandler: MessageHandler
    ) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    alias(): string {
        return this.meta.alias;
    }

    async collectOutputs(outputIds: string[]): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'collectOutputs',
                    data: {
                        outputIdsToCollect: outputIds
                    },
                },
            },
        })
    }

    async getOutputsWithAdditionalUnlockConditions(outputs: OutputsToCollect): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'getOutputsWithAdditionalUnlockConditions',
                    data: {
                        outputsToCollect: outputs
                    },
                },
            },
        })
    }

    async listAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listAddresses'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listAddressesWithBalance(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listAddressesWithBalance'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listOutputs'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listUnspentOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listUnspentOutputs'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listPendingTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listPendingTransactions'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'listTransactions'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sync(options: AccountSyncOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'syncAccount',
                    data: options || {},
                },
            },
        })
    }

    /**
     * TODO: Test this method through example and see if the interface is correct
     */
    async getNodeInfo(url: string): Promise<NodeInfo> {
        return JSON.parse(
            await this.messageHandler.sendMessage({
                cmd: 'callAccountMethod',
                payload: {
                    accountId: this.meta.index,
                    method: {
                        name: 'getNodeInfo',
                        data: [url],
                    },
                },
            }),
        );
    }

    async generateAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'generateAddresses',
                    data: {
                        // TODO: Why is the amount set to 1 here?
                        amount: 1,
                    }
                },
            },
        });

        return JSON.parse(response).payload;
    }

    async latestAddress(): Promise<Address> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'getLatestAddress',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    async balance(): Promise<AccountBalance> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'getBalance',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    async mintNativeToken(nativeTokenOptions: NativeTokenOptions, transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'mintNativeToken',
                    data: {
                        nativeTokenOptions: nativeTokenOptions,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async mintNfts(nftOptions: NftOptions, transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'mintNfts',
                    data: {
                        nftsOptions: nftOptions,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendAmount(addressesWithAmount: AddressWithAmount[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'sendAmount',
                    data: {
                        addressesWithAmount,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendMicroTransaction(addressesWithMicroAmount: AddressMicroAmount[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'sendMicroTransaction',
                    data: {
                        addressesWithMicroAmount: addressesWithMicroAmount,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendNativeTokens(addressNativeTokens: AddressNativeTokens[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'sendNativeTokens',
                    data: {
                        addressesNativeTokens: addressNativeTokens,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendNft(addressesAndNftIds: AddressNftId[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'sendNft',
                    data: {
                        addressesNftIds: addressesAndNftIds,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendTransfer(outputs: OutputData[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'sendTransfer',
                    data: {
                        outputs,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async tryCollectOutputs(outputsToCollect: OutputsToCollect): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'callAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'tryCollectOutputs',
                    data: {
                        outputsToCollect
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }


    /**
     * TODO: Replace any with proper response type
     */
    async setClientOptions(options: ClientOptions): Promise<any> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'setClientOptions',
            payload: options,
        });

        return JSON.parse(response).payload;
    }
}
