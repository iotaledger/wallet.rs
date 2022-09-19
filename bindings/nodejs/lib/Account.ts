// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { MessageHandler } from './MessageHandler';
import type {
    AccountBalance,
    AccountMetadata,
    AccountSyncOptions,
    AccountMeta,
    Address,
    AddressWithAmount,
    AddressWithMicroAmount,
    AddressNativeTokens,
    AddressNftId,
    AddressGenerationOptions,
    AddressWithUnspentOutputs,
    FilterOptions,
    IncreaseNativeTokenSupplyOptions,
    MintTokenTransaction,
    NativeTokenOptions,
    NftOptions,
    OutputData,
    OutputOptions,
    OutputsToClaim,
    PreparedTransactionData,
    Transaction,
    TransactionOptions,
} from '../types';
import type { SignedTransactionEssence } from '../types/signedTransactionEssence';
import type {
    BuildAliasOutputData,
    BuildBasicOutputData,
    BuildFoundryOutputData,
    BuildNftOutputData,
} from '../types/buildOutputData';
import type {
    HexEncodedAmount,
    IAliasOutput,
    IBasicOutput,
    IFoundryOutput,
    INftOutput,
    IOutputResponse,
    ITransactionPayload,
    OutputTypes,
} from '@iota/types';

/** The Account class. */
export class Account {
    // private because the data isn't updated
    private meta: AccountMeta;
    private messageHandler: MessageHandler;

    constructor(accountMeta: AccountMeta, messageHandler: MessageHandler) {
        this.meta = accountMeta;
        this.messageHandler = messageHandler;
    }

    /**
     * Build an `AliasOutput`.
     * @param data Options for building an `AliasOutput`.
     * @returns The built `AliasOutput`.
     */
    async buildAliasOutput(data: BuildAliasOutputData): Promise<IAliasOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildAliasOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Build a `BasicOutput`.
     * @param data Options for building a `BasicOutput`.
     * @returns The built `BasicOutput`.
     */
    async buildBasicOutput(data: BuildBasicOutputData): Promise<IBasicOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildBasicOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Build a `FoundryOutput`.
     * @param data Options for building a `FoundryOutput`.
     * @returns The built `FoundryOutput`.
     */
    async buildFoundryOutput(
        data: BuildFoundryOutputData,
    ): Promise<IFoundryOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildFoundryOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Build an `NftOutput`.
     * @param data Options for building an `NftOutput`.
     * @returns The built `NftOutput`.
     */
    async buildNftOutput(data: BuildNftOutputData): Promise<INftOutput> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BuildNftOutput',
                data,
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
     * the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
     * recommended to use melting, if the foundry output is available.
     * @param tokenId The native token id.
     * @param burnAmount The to be burned amount.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The transaction.
     */
    async burnNativeToken(
        tokenId: string,
        burnAmount: HexEncodedAmount,
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BurnNativeToken',
                data: {
                    tokenId,
                    burnAmount,
                    options: transactionOptions,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Burn an nft output. Outputs controlled by it will be sweeped before if they don't have a storage
     * deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
     * burning, the foundry can never be destroyed anymore.
     * @param nftId The NftId.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The transaction.
     */
    async burnNft(
        nftId: string,
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'BurnNft',
                data: {
                    nftId,
                    options: transactionOptions,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Claim basic or nft outputs that have additional unlock conditions
     * to their `AddressUnlockCondition` from the account.
     * @param outputIds The outputs to claim.
     * @returns The resulting transaction.
     */
    async claimOutputs(outputIds: string[]): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ClaimOutputs',
                data: {
                    outputIdsToClaim: outputIds,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Consolidate basic outputs with only an `AddressUnlockCondition` from an account
     * by sending them to an own address again if the output amount is greater or
     * equal to the output consolidation threshold.
     * @param force Force consolidation on addresses where the threshold isn't met.
     * @param outputConsolidationThreshold A default threshold is used if this is omitted.
     * @returns The consolidation transaction.
     */
    async consolidateOutputs(
        force: boolean,
        outputConsolidationThreshold?: number,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ConsolidateOutputs',
                data: {
                    force,
                    outputConsolidationThreshold,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Melt native tokens. This happens with the foundry output which minted them, by increasing its
     * `melted_tokens` field.
     * @param tokenId The native token id.
     * @param meltAmount To be melted amount.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The transaction.
     */
    async decreaseNativeTokenSupply(
        tokenId: string,
        meltAmount: HexEncodedAmount,
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'DecreaseNativeTokenSupply',
                data: {
                    tokenId,
                    meltAmount,
                    options: transactionOptions,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Destroy an alias output. Outputs controlled by it will be sweeped before if they don't have a
     * storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
     * sent to the governor address.
     * @param aliasId The AliasId.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The transaction.
     */
    async destroyAlias(
        aliasId: string,
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'DestroyAlias',
                data: {
                    aliasId,
                    options: transactionOptions,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Function to destroy a foundry output with a circulating supply of 0.
     * Native tokens in the foundry (minted by other foundries) will be transactioned to the controlling alias.
     * @param foundryId The FoundryId.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The transaction.
     */
    async destroyFoundry(
        foundryId: string,
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'DestroyFoundry',
                data: {
                    foundryId,
                    options: transactionOptions,
                },
            },
        );
        return JSON.parse(resp).payload;
    }

    /**
     * Generate a new unused address.
     * @param options Options for address generation.
     * @returns The address.
     */
    async generateAddress(
        options?: AddressGenerationOptions,
    ): Promise<Address> {
        const addresses = await this.generateAddresses(1, options);
        return addresses[0];
    }

    /**
     * Generate new unused addresses.
     * @param amount The amount of addresses to generate.
     * @param options Options for address generation.
     * @returns The addresses.
     */
    async generateAddresses(
        amount: number,
        options?: AddressGenerationOptions,
    ): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GenerateAddresses',
                data: {
                    amount,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Get the account balance.
     * @returns The account balance.
     */
    async getBalance(): Promise<AccountBalance> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetBalance',
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Get the data for an output.
     * @param outputId The output to get.
     * @returns The `OutputData`.
     */
    async getOutput(outputId: string): Promise<OutputData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetOutput',
                data: {
                    outputId,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Get a `FoundryOutput` by native token ID. It will try to get the foundry from
     * the account, if it isn't in the account it will try to get it from the node.
     * @param tokenId The native token ID to get the foundry for.
     * @returns The `FoundryOutput` that minted the token.
     */
    async getFoundryOutput(tokenId: string): Promise<IFoundryOutput> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetFoundryOutput',
                data: {
                    tokenId,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Get outputs with additional unlock conditions.
     * @param outputs The type of outputs to claim.
     * @returns The output IDs of the unlockable outputs.
     */
    async getOutputsWithAdditionalUnlockConditions(
        outputs: OutputsToClaim,
    ): Promise<string[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetOutputsWithAdditionalUnlockConditions',
                data: {
                    outputsToClaim: outputs,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Get a transaction stored in the account.
     * @param transactionId The ID of the transaction to get.
     * @returns The transaction.
     */
    async getTransaction(transactionId: string): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'GetTransaction',
                data: {
                    transactionId,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * List all the addresses of the account.
     * @returns The addresses.
     */
    async listAddresses(): Promise<Address[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListAddresses',
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * List the addresses of the account with unspent outputs.
     * @returns The addresses.
     */
    async listAddressesWithUnspentOutputs(): Promise<
        AddressWithUnspentOutputs[]
    > {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListAddressesWithUnspentOutputs',
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * List all outputs of the account.
     * @param filterOptions Options to filter the to be returned outputs.
     * @returns The outputs with metadata.
     */
    async listOutputs(filterOptions?: FilterOptions): Promise<OutputData[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListOutputs',
                data: { filterOptions },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * List all the pending transactions of the account.
     * @returns The transactions.
     */
    async listPendingTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListPendingTransactions',
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * List all incoming transactions of the account.
     * @returns The incoming transactions with their inputs.
     */
    async listIncomingTransactions(): Promise<
        [string, ITransactionPayload, IOutputResponse][]
    > {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListIncomingTransactions',
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * List all the transactions of the account.
     * @returns The transactions.
     */
    async listTransactions(): Promise<Transaction[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListTransactions',
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * List all the unspent outputs of the account.
     * @param filterOptions Options to filter the to be returned outputs.
     * @returns The outputs with metadata.
     */
    async listUnspentOutputs(
        filterOptions?: FilterOptions,
    ): Promise<OutputData[]> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'ListUnspentOutputs',
                data: { filterOptions },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Get the accounts metadata.
     * @returns The accounts metadata.
     */
    getMetadata(): AccountMetadata {
        return {
            alias: this.meta.alias,
            coinType: this.meta.coinType,
            index: this.meta.index,
        };
    }

    /**
     * Calculate the minimum required storage deposit for an output.
     * @param output output to calculate the deposit amount for.
     * @returns The amount.
     */
    async minimumRequiredStorageDeposit(output: OutputTypes): Promise<string> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MinimumRequiredStorageDeposit',
                data: {
                    output,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Mint more native tokens.
     * @param tokenId The native token id.
     * @param mintAmount To be minted amount.
     * @param increaseNativeTokenSupplyOptions Options for minting more tokens.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The minting transaction and the token ID.
     */
    async increaseNativeTokenSupply(
        tokenId: string,
        mintAmount: HexEncodedAmount,
        increaseNativeTokenSupplyOptions?: IncreaseNativeTokenSupplyOptions,
        transactionOptions?: TransactionOptions,
    ): Promise<MintTokenTransaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'IncreaseNativeTokenSupply',
                data: {
                    tokenId,
                    mintAmount,
                    increaseNativeTokenSupplyOptions,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Mint native tokens.
     * @param nativeTokenOptions The options for minting tokens.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The minting transaction and the token ID.
     */
    async mintNativeToken(
        nativeTokenOptions: NativeTokenOptions,
        transactionOptions?: TransactionOptions,
    ): Promise<MintTokenTransaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNativeToken',
                data: {
                    nativeTokenOptions: nativeTokenOptions,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Mint nfts.
     * @param nftsOptions The options for minting nfts.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The minting transaction.
     */
    async mintNfts(
        nftsOptions: NftOptions[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'MintNfts',
                data: {
                    nftsOptions,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Prepare an output for sending, useful for offline signing.
     * @param options The options for preparing an output. If the amount is
     * below the minimum required storage deposit, by default the remaining
     * amount will automatically be added with a `StorageDepositReturn` `UnlockCondition`,
     * when setting the `ReturnStrategy` to `gift`, the full minimum required
     * storage deposit will be sent  to the recipient. When the assets contain
     * an nft id, the data from the existing `NftOutput` will be used, just with
     * the address unlock conditions replaced.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The prepared output.
     */
    async prepareOutput(
        options: OutputOptions,
        transactionOptions?: TransactionOptions,
    ): Promise<OutputTypes> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareOutput',
                data: {
                    options,
                    transactionOptions,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Prepare a send amount transaction, useful for offline signing.
     * @param addressesWithAmount Address with amounts to send.
     * @param options The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The prepared transaction data.
     */
    async prepareSendAmount(
        addressesWithAmount: AddressWithAmount[],
        options?: TransactionOptions,
    ): Promise<PreparedTransactionData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareSendAmount',
                data: {
                    addressesWithAmount,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Prepare a transaction, useful for offline signing.
     * @param outputs Outputs to use in the transaction.
     * @param options The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The prepared transaction data.
     */
    async prepareTransaction(
        outputs: OutputTypes[],
        options?: TransactionOptions,
    ): Promise<PreparedTransactionData> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'PrepareTransaction',
                data: {
                    outputs,
                    options,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Send a transaction with amounts from input addresses.
     * @param addressesWithAmount Addresses with amounts.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The sent transaction.
     */
    async sendAmount(
        addressesWithAmount: AddressWithAmount[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendAmount',
                data: {
                    addressesWithAmount,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Send a micro transaction with amount below minimum storage deposit.
     * @param addressesWithMicroAmount Addresses with micro amounts.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The sent transaction.
     */
    async sendMicroTransaction(
        addressesWithMicroAmount: AddressWithMicroAmount[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendMicroTransaction',
                data: {
                    addressesWithMicroAmount,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Send native tokens.
     * @param addressesNativeTokens Addresses amounts and native tokens.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The sent transaction.
     */
    async sendNativeTokens(
        addressesNativeTokens: AddressNativeTokens[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNativeTokens',
                data: {
                    addressesNativeTokens,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Send nft.
     * @param addressesAndNftIds Addresses and nft ids.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The sent transaction.
     */
    async sendNft(
        addressesAndNftIds: AddressNftId[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendNft',
                data: {
                    addressesAndNftIds,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    /**
     * Send outputs in a transaction.
     * @param outputs The outputs to send.
     * @param transactionOptions The options to define a `RemainderValueStrategy`
     * or custom inputs.
     * @returns The sent transaction.
     */
    async sendOutputs(
        outputs: OutputTypes[],
        transactionOptions?: TransactionOptions,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SendOutputs',
                data: {
                    outputs,
                    options: transactionOptions,
                },
            },
        );

        return JSON.parse(response).payload;
    }

    async setAlias(alias: string): Promise<void> {
        await this.messageHandler.callAccountMethod(this.meta.index, {
            name: 'SetAlias',
            data: {
                alias,
            },
        });
    }

    /**
     * Sign a prepared transaction, useful for offline signing.
     * @param preparedTransactionData The prepared transaction data to sign.
     * @returns The signed transaction essence.
     */
    async signTransactionEssence(
        preparedTransactionData: PreparedTransactionData,
    ): Promise<SignedTransactionEssence> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SignTransactionEssence',
                data: {
                    preparedTransactionData,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Validate the transaction, submit it to a node and store it in the account.
     * @param signedTransactionData A signed transaction to submit and store.
     * @returns The sent transaction.
     */
    async submitAndStoreTransaction(
        signedTransactionData: SignedTransactionEssence,
    ): Promise<Transaction> {
        const response = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SubmitAndStoreTransaction',
                data: {
                    signedTransactionData,
                },
            },
        );
        return JSON.parse(response).payload;
    }

    /**
     * Sync the account by fetching new information from the nodes.
     * Will also retry pending transactions if necessary.
     * @param options Optional synchronization options.
     * @returns The account balance.
     */
    async sync(options?: AccountSyncOptions): Promise<AccountBalance> {
        const resp = await this.messageHandler.callAccountMethod(
            this.meta.index,
            {
                name: 'SyncAccount',
                data: {
                    options,
                },
            },
        );
        return JSON.parse(resp).payload;
    }
}
