export enum Network {
    Mainnet,
    Testnet,
}

export type Auth = {
    jwt?: string;
    username?: string;
    password?: string;
}

export interface MqttBrokerOptions {
    automaticDisconnect?: boolean;
    // timeout in seconds
    timeout?: number;
    useWs?: boolean;
    port?: number;
    maxReconnectionAttempts?: number;
}

export type Node = {
    url: string;
    auth?: Auth;
    disabled?: boolean;
}

export interface ClientOptions {
    primaryNode?: string | Node;
    primaryPoWNode?: string | Node;
    node?: string | Node;
    nodes?: Array<string | Node>;
    network?: string;
    mqttBrokerOptions?: MqttBrokerOptions;
    quorumSize?: number;
    quorumThreshold?: number;
    localPow?: boolean;
}

export interface NodeInfo {
    name: string;
    version: string;
    isHealthy: boolean;
    networkId: string;
    bech32HRP: string;
    minPoWScore: number;
    messagesPerSecond: number;
    referencedMessagesPerSecond: number;
    referencedRate: number;
    latestMilestoneTimestamp: number;
    latestMilestoneIndex: number;
    confirmedMilestoneIndex: number;
    pruningIndex: number;
    features: string[];
}
