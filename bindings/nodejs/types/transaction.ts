// TODO: Update Transaction interface
export interface Transaction {
    // TODO: Change to camelCase
    message_id: string;
    // TODO: Add other inclusion states
    inclusion_state: 'Confirmed';
    timestamp: number;
    // TODO: Change to camelCase
    network_id: number;
    incoming: boolean;
}
