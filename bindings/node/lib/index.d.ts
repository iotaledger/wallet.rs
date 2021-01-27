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

export declare interface TransactionEssence {
  inputs: Input[];
  outputs: Output[];
  payload?: Payload[];
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
  essence: TransactionEssence;
}

export declare type Payload = Transaction;

export declare interface Message {
  version: number;
  trunk: string;
  branch: string;
  payload_length: number;
  payload: Payload;
  timestamp: string;
  nonce: number;
  confirmed: boolean;
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

export declare class Account {
  id(): string;
  index(): number;
  alias(): string;
  availableBalance(): number;
  totalBalance(): number;
  listMessages(count?: number, from?: number, messageType?: MessageType): Message[]
  listAddresses(unspent?: boolean): Address[]
  sync(options?: SyncOptions): Promise<SyncedAccount>
  setAlias(alias: string): void
  setClientOptions(options: ClientOptions): void
  getMessage(id: string): Message | undefined
  generateAddress(): Address
  latestAddress(): Address | undefined
}

export declare class RemainderValueStrategy {
  static changeAddress(): RemainderValueStrategy
  static reuseAddress(): RemainderValueStrategy
  static accountAddress(address: string): RemainderValueStrategy
}

export declare class SyncedAccount {
  send(address: string, amount: number, remainderValueStrategy?: RemainderValueStrategy): Promise<Message>
  retry(messageId: string): Promise<Message>
  reattach(messageId: string): Promise<Message>
  promote(messageId: string): Promise<Message>
}

export declare enum Network {
  Mainnet,
  Devnet,
  Comnet
}

export declare interface ClientOptions {
  node?: string;
  nodes?: string[];
  network?: Network;
  quorumSize?: number;
  quorumThreshold?: number;
}

export declare interface AccountToCreate {
  clientOptions: ClientOptions;
  mnemonic?: string;
  alias?: string;
  createdAt?: string;
}

export declare class AccountManager {
  constructor(storagePath?: string)
  setStrongholdPassword(password: string): void
  createAccount(account: AccountToCreate): Account
  getAccount(accountId: string | number): Account | undefined
  getAccountByAlias(alias: string): Account | undefined
  getAccounts(): Account[]
  removeAccount(accountId: string | number): void
  syncAccounts(): Promise<SyncedAccount[]>
  internalTransfer(fromAccount: Account, toAccount: Account, amount: number): Promise<Message>
  backup(destination: string): string
  importAccounts(source: string): void
}

export declare type Event = 'ErrorThrown' |
  'BalanceChange' |
  'NewTransaction' |
  'ConfirmationStateChange' |
  'Reattachment' |
  'Broadcast'

export declare function addEventListener(event: Event, cb: (err?: any, data?: { [k: string]: any }) => void): void
