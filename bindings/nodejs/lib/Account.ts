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
    AddressWithMicroAmount,
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'CollectOutputs',
                    data: {
                        outputIdsToCollect: outputIds
                    },
                },
            },
        })
    }

    async getOutputsWithAdditionalUnlockConditions(outputs: OutputsToCollect): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'GetOutputsWithAdditionalUnlockConditions',
                    data: {
                        outputsToCollect: outputs
                    },
                },
            },
        })
    }

    async listAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListAddresses'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listAddressesWithBalance(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListAddressesWithBalance'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListOutputs'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listUnspentOutputs(): Promise<OutputData[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListUnspentOutputs'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listPendingTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListPendingTransactions'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async listTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'ListTransactions'
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sync(options: AccountSyncOptions): Promise<void> {
        await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SyncAccount',
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
                cmd: 'CallAccountMethod',
                payload: {
                    accountId: this.meta.index,
                    method: {
                        name: 'GetNodeInfo',
                        data: [url],
                    },
                },
            }),
        );
    }

    async generateAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'GenerateAddresses',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'GetLatestAddress',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    async balance(): Promise<AccountBalance> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'GetBalance',
                },
            },
        });

        return JSON.parse(response).payload;
    }

    async mintNativeToken(nativeTokenOptions: NativeTokenOptions, transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'MintNativeToken',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'MintNfts',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SendAmount',
                    data: {
                        addressesWithAmount,
                        options: transferOptions
                    }
                },
            },
        })

        return JSON.parse(response).payload;
    }

    async sendMicroTransaction(addressesWithMicroAmount: AddressWithMicroAmount[], transferOptions: TransferOptions): Promise<Transaction[]> {
        const response = await this.messageHandler.sendMessage({
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SendMicroTransaction',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SendNativeTokens',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SendNft',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'SendTransfer',
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
            cmd: 'CallAccountMethod',
            payload: {
                accountId: this.meta.index,
                method: {
                    name: 'TryCollectOutputs',
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
            cmd: 'SetClientOptions',
            payload: options,
        });

        return JSON.parse(response).payload;
    }
}
