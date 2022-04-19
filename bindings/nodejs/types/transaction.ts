// TODO: Update Transaction interface
export interface Transaction {
    messageId: string;
    inclusionState: 'Confirmed';
    timestamp: number;
    networkId: number;
    incoming: boolean;
}
