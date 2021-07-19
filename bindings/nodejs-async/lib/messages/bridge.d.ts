// import type { Account, AccountIdentifier, Balance, SyncedAccount } from './account';
// import type { Address } from './address';
// import type { ErrorEventPayload } from './events';
// import type { Message } from './message';
// import type { NodeInfo } from './node';
// import type { MigrationBundle, MigrationData, SendMigrationBundleResponse } from './migration';
// import type { StrongholdStatus } from './wallet';
export interface CommunicationIds {
    messageId: string;
}
export interface BridgeMessage {
    id: string;
    cmd: string;
    payload?: any;
}
export declare enum ResponseTypes {
    InvalidMessage = "InvalidMessage",
    RemovedAccount = "RemovedAccount",
    CreatedAccount = "CreatedAccount",
    ReadAccount = "ReadAccount",
    ReadAccounts = "ReadAccounts",
    Messages = "Messages",
    Addresses = "Addresses",
    GeneratedAddress = "GeneratedAddress",
    LatestAddress = "LatestAddress",
    Balance = "Balance",
    SyncedAccounts = "SyncedAccounts",
    SyncedAccount = "SyncedAccount",
    Reattached = "Reattached",
    BackupSuccessful = "BackupSuccessful",
    BackupRestored = "BackupRestored",
    StrongholdPasswordSet = "StrongholdPasswordSet",
    SentTransfer = "SentTransfer",
    Error = "Error",
    Panic = "Panic",
    ErrorThrown = "ErrorThrown",
    BalanceChange = "BalanceChange",
    NewTransaction = "NewTransaction",
    ConfirmationStateChange = "ConfirmationStateChange",
    Reattachment = "Reattachment",
    Broadcast = "Broadcast",
    StrongholdStatusChange = "StrongholdStatusChange",
    TransferProgress = "TransferProgress",
    MigrationProgress = "MigrationProgress",
    GeneratedMnemonic = "GeneratedMnemonic",
    StoredMnemonic = "StoredMnemonic",
    VerifiedMnemonic = "VerifiedMnemonic",
    StoragePasswordSet = "StoragePasswordSet",
    StrongholdStatus = "StrongholdStatus",
    UnusedAddress = "UnusedAddress",
    IsLatestAddressUnused = "IsLatestAddressUnused",
    AreAllLatestAddressesUnused = "AreAllLatestAddressesUnused",
    UpdatedAlias = "UpdatedAlias",
    DeletedStorage = "DeletedStorage",
    LockedStronghold = "LockedStronghold",
    StrongholdPasswordChanged = "StrongholdPasswordChanged",
    UpdatedAllClientOptions = "UpdatedAllClientOptions",
    StrongholdPasswordClearIntervalSet = "StrongholdPasswordClearIntervalSet",
    MigrationData = "MigrationData",
    CreatedMigrationBundle = "CreatedMigrationBundle",
    SentMigrationBundle = "SentMigrationBundle",
    LegacySeedChecksum = "SeedChecksum",
    NodeInfo = "NodeInfo"
}
export declare enum Actions {
    RemoveAccount = "RemoveAccount"
}
export declare type Response<T, P> = {
    id: string;
    action: Actions;
    type: T;
    payload?: P;
};
// export declare type RemovedAccountResponse = Response<ResponseTypes.RemovedAccount, AccountIdentifier>;
// export declare type CreatedAccountResponse = Response<ResponseTypes.CreatedAccount, Account>;
// export declare type ReadAccountResponse = Response<ResponseTypes.ReadAccount, Account>;
// export declare type ReadAccountsResponse = Response<ResponseTypes.ReadAccounts, Account[]>;
// export declare type ListMessagesResponse = Response<ResponseTypes.Messages, Message[]>;
// export declare type ListAddressesResponse = Response<ResponseTypes.Addresses, Address[]>;
// export declare type GeneratedAddressResponse = Response<ResponseTypes.GeneratedAddress, Address>;
// export declare type LatestAddressResponse = Response<ResponseTypes.LatestAddress, Address>;
// export declare type BalanceResponse = Response<ResponseTypes.Balance, Balance>;
// export declare type SyncAccountsResponse = Response<ResponseTypes.SyncedAccounts, SyncedAccount[]>;
// export declare type SyncAccountResponse = Response<ResponseTypes.SyncedAccount, SyncedAccount>;
export declare type ReattachResponse = Response<ResponseTypes.Reattached, string>;
export declare type BackupSuccessfulResponse = Response<ResponseTypes.BackupSuccessful, void>;
export declare type BackupRestoredResponse = Response<ResponseTypes.BackupRestored, void>;
export declare type SetStrongholdPasswordResponse = Response<ResponseTypes.StrongholdPasswordSet, void>;
// export declare type SentTransferResponse = Response<ResponseTypes.SentTransfer, Message>;
// export declare type ErrorResponse = Response<ResponseTypes.Error, ErrorEventPayload>;
export declare type PanicResponse = Response<ResponseTypes.Panic, string>;
export declare type GenerateMnemonicResponse = Response<ResponseTypes.GeneratedMnemonic, string>;
export declare type StoreMnemonicResponse = Response<ResponseTypes.StoredMnemonic, void>;
export declare type VerifyMnemonicResponse = Response<ResponseTypes.VerifiedMnemonic, void>;
export declare type SetStoragePasswordResponse = Response<ResponseTypes.StoragePasswordSet, void>;
// export declare type StrongholdStatusResponse = Response<ResponseTypes.StrongholdStatus, StrongholdStatus>;
// export declare type UnusedAddressResponse = Response<ResponseTypes.UnusedAddress, Address>;
export declare type IsLatestAddressUnusedResponse = Response<ResponseTypes.IsLatestAddressUnused, boolean>;
export declare type AreLatestAddressesUnusedResponse = Response<ResponseTypes.AreAllLatestAddressesUnused, boolean>;
export declare type SetAliasResponse = Response<ResponseTypes.UpdatedAlias, void>;
export declare type DeleteStorageResponse = Response<ResponseTypes.DeletedStorage, void>;
export declare type LockStrongholdResponse = Response<ResponseTypes.LockedStronghold, void>;
export declare type StrongholdPasswordChangeResponse = Response<ResponseTypes.StrongholdPasswordChanged, void>;
export declare type UpdatedAllClientOptions = Response<ResponseTypes.UpdatedAllClientOptions, void>;
export declare type LegacySeedChecksum = Response<ResponseTypes.LegacySeedChecksum, string>;
/**
 * Migration responses
 */
// export declare type MigrationDataResponse = Response<ResponseTypes.MigrationData, MigrationData>;
// export declare type CreatedMigrationBundleResponse = Response<ResponseTypes.CreatedMigrationBundle, MigrationBundle>;
// export declare type SentMigrationBundleResponse = Response<ResponseTypes.SentMigrationBundle, SendMigrationBundleResponse>;
// export declare type GetNodeInfoResponse = Response<ResponseTypes.NodeInfo, NodeInfo>;
// export declare type MessageResponse = RemovedAccountResponse | CreatedAccountResponse | ReadAccountResponse | ReadAccountsResponse | ListMessagesResponse | ListAddressesResponse | GeneratedAddressResponse | LatestAddressResponse | BalanceResponse | SyncAccountsResponse | SyncAccountResponse | ReattachResponse | BackupSuccessfulResponse | BackupRestoredResponse | SetStrongholdPasswordResponse | SentTransferResponse | ErrorResponse | PanicResponse | GenerateMnemonicResponse | StoreMnemonicResponse | VerifyMnemonicResponse | SetStoragePasswordResponse | StrongholdStatusResponse | UnusedAddressResponse | IsLatestAddressUnusedResponse | AreLatestAddressesUnusedResponse | SetAliasResponse | DeleteStorageResponse | LockStrongholdResponse | UpdatedAllClientOptions | LegacySeedChecksum | MigrationDataResponse | CreatedMigrationBundleResponse | SentMigrationBundleResponse | GetNodeInfoResponse;
export declare type Bridge = (message: BridgeMessage) => Promise<string>;
