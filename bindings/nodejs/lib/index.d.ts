export declare enum MessageType {
  /// Message received.
  Received = 1,
  /// Message sent.
  Sent = 2,
  /// Message not broadcasted.
  Failed = 3,
  /// Message not confirmed.
  Unconfirmed = 4,
  /// A value message.
  Value = 5,
}

export declare interface RegularEssence {
  inputs: Input[];
  outputs: Output[];
  payload?: Payload[];
}

export declare type Essence = {
  type: 'Regular',
  data: RegularEssence
}

export declare interface Input {
  transactionId: string
  outputIndex: number
}

export declare interface Output {
  address: string
  amount: number
}

export declare interface Transaction {
  essence: Essence;
}

export declare type Payload = Transaction;

export declare interface Message {
  version: number;
  parents: string[];
  payloadLength: number;
  payload: Payload;
  timestamp: string;
  nonce: number;
  confirmed?: boolean;
  broadcasted: boolean;
  incoming: boolean;
  value: number;
}

export declare interface Address {
  address: string;
  balance: number;
  keyIndex: number;
}

export declare interface SyncOptions {
  addressIndex?: number
  gapLimit?: number
  skipPersistance?: boolean
}

export declare interface AccountBalance {
  total: number
  available: number
  incoming: number
  outgoing: number
}

export declare class Account {
  id(): string;
  index(): number;
  alias(): string;
  balance(): AccountBalance;
  listMessages(count?: number, from?: number, messageType?: MessageType): Message[]
  listAddresses(unspent?: boolean): Address[]
  sync(options?: SyncOptions): Promise<SyncedAccount>
  setAlias(alias: string): void
  setClientOptions(options: ClientOptions): void
  getMessage(id: string): Message | undefined
  getAddress(addressBech32: string): Address | undefined
  generateAddress(): Address
  latestAddress(): Address
  getUnusedAddress(): Address
  isLatestAddressUnused(): Promise<boolean>
}

export declare class RemainderValueStrategy {
  static changeAddress(): RemainderValueStrategy
  static reuseAddress(): RemainderValueStrategy
  static accountAddress(address: string): RemainderValueStrategy
}

export declare class TransferOptions {
  remainderValueStrategy?: RemainderValueStrategy
  indexation?: { index: string | number[] | Uint8Array, data?: string | number[] | Uint8Array }
}

export declare class SyncedAccount {
  send(address: string, amount: number, options?: TransferOptions): Promise<Message>
  retry(messageId: string): Promise<Message>
  reattach(messageId: string): Promise<Message>
  promote(messageId: string): Promise<Message>
  consolidateOutputs(): Promise<Message[]>
}

export declare interface ClientOptions {
  node?: string;
  nodes?: string[];
  network?: string;
  quorumSize?: number;
  quorumThreshold?: number;
  localPow?: boolean;
}

export declare enum SignerType {
  Stronghold = 1
}

export declare interface AccountToCreate {
  clientOptions: ClientOptions;
  mnemonic?: string;
  alias?: string;
  createdAt?: string;
  signerType?: SignerType;
  skipPersistance?: boolean;
}

export declare enum StorageType {
  Sqlite,
  Stronghold
}

export declare interface ManagerOptions {
  storagePath?: string
  storageType?: StorageType
  storagePassword?: string
  outputConsolidationThreshold?: number
  automaticOutputConsolidation?: boolean
}

export declare class AccountManager {
  constructor(options: ManagerOptions)
  setStoragePassword(password: string): void
  setStrongholdPassword(password: string): void
  changeStrongholdPassword(currentPassword: string, newPassword: string): void
  generateMnemonic(): string
  storeMnemonic(signerType: SignerType, mnemonic?: string): void
  createAccount(account: AccountToCreate): Account
  getAccount(accountId: string | number): Account | undefined
  getAccounts(): Account[]
  removeAccount(accountId: string | number): void
  syncAccounts(): Promise<SyncedAccount[]>
  internalTransfer(fromAccount: Account, toAccount: Account, amount: number): Promise<Message>
  backup(destination: string): string
  importAccounts(source: string, password: string): void
  isLatestAddressUnused(): Promise<boolean>
  setClientOptions(options: ClientOptions): void
}

export declare type Event = 'ErrorThrown' |
  'BalanceChange' |
  'NewTransaction' |
  'ConfirmationStateChange' |
  'Reattachment' |
  'Broadcast'

export interface LoggerOutput {
  name?: string
  level_filter: 'off' | 'error' | 'warn' | 'info' | 'debug' | 'trace'
  target_filters?: string[]
}

export interface LoggerConfig {
  color_enabled?: boolean
  outputs?: LoggerOutput[]
}

export declare function addEventListener(event: Event, cb: (err?: any, data?: { [k: string]: any }) => void): void
export declare function initLogger(config: LoggerConfig)