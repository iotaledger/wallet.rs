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
  /// Message confirmed.
  Confirmed = 6,
}

export declare interface RegularEssence {
  inputs: Input[];
  outputs: Output[];
  payload?: Payload[];
  incoming: boolean;
  internal: boolean;
  value: number;
  remainderValue: number;
}

export declare type Essence = {
  type: 'Regular',
  data: RegularEssence
}

export declare interface UTXOInput {
  input: string
  metadata?: {
    transactionId: string
    messageId: string
    index: number
    amount: number
    is_spent: boolean
    address: string
  }
}

export declare type Input = { type: 'UTXO', data: UTXOInput }

export declare interface SignatureLockedSingleOutput {
  address: string
  amount: number
  remainder: boolean
}

export declare interface SignatureLockedDustAllowance {
  address: string
  amount: number
  remainder: boolean
}

export declare type Output = {
  type: 'SignatureLockedSingleOutput',
  data: SignatureLockedSingleOutput
}
  | {
    type: 'SignatureLockedDustAllowance',
    data: SignatureLockedDustAllowance
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
}

export declare interface Address {
  address: string;
  balance: number;
  keyIndex: number;
}

export declare interface SyncOptions {
  addressIndex?: number
  gapLimit?: number
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
  messageCount(messageType?: MessageType): number;
  listMessages(count?: number, from?: number, messageType?: MessageType): Message[]
  listAddresses(unspent?: boolean): Address[]
  sync(options?: SyncOptions): Promise<SyncedAccount>
  send(address: string, amount: number, options?: TransferOptions): Promise<Message>
  retry(messageId: string): Promise<Message>
  reattach(messageId: string): Promise<Message>
  promote(messageId: string): Promise<Message>
  consolidateOutputs(): Promise<Message[]>
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

export declare class SyncedAccount { }

export declare type NodeUrl = string

export declare interface Node {
  url: NodeUrl
  auth?: {
    username: string
    password: string
  }
}

export declare interface ClientOptions {
  node?: NodeUrl | Node;
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
  skipPersistence?: boolean;
}

export declare interface ManagerOptions {
  storagePath?: string
  storagePassword?: string
  outputConsolidationThreshold?: number
  automaticOutputConsolidation?: boolean
  syncSpentOutputs?: boolean
  persistEvents?: boolean
}

export declare interface BalanceChangeEvent {
  indexationId: string
  accountId: string
  address: string
  messageId?: string
  balanceChange: { spent: number, received: number }
}

export declare interface TransactionConfirmationEvent {
  indexationId: string
  accountId: string
  message: Message
  confirmed: boolean
}

export declare interface TransactionEvent {
  indexationId: string
  accountId: string
  message: Message
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
  syncAccounts(options?: SyncOptions): Promise<SyncedAccount[]>
  internalTransfer(fromAccount: Account, toAccount: Account, amount: number): Promise<Message>
  backup(destination: string): string
  importAccounts(source: string, password: string): void
  isLatestAddressUnused(): Promise<boolean>
  setClientOptions(options: ClientOptions): void
  // events
  getBalanceChangeEvents(count?: number, skip?: number, fromTimestamp?: number): BalanceChangeEvent[]
  getBalanceChangeEventCount(fromTimestamp?: number): number
  getTransactionConfirmationEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionConfirmationEvent[]
  getTransactionConfirmationEventCount(fromTimestamp?: number): number
  getNewTransactionEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
  getNewTransactionEventCount(fromTimestamp?: number): number
  getReattachmentEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
  getReattachmentEventCount(fromTimestamp?: number): number
  getBroadcastEvents(count?: number, skip?: number, fromTimestamp?: number): TransactionEvent[]
  getBroadcastEventCount(fromTimestamp?: number): number
}

export declare type Event = 'ErrorThrown' |
  'BalanceChange' |
  'NewTransaction' |
  'ConfirmationStateChange' |
  'Reattachment' |
  'Broadcast' |
  'TransferProgress'

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