import type { INodeInfo } from '@iota/types';

export enum Network {
    Mainnet,
    Testnet,
}

type Auth = {
    jwt?: string;
    username?: string;
    password?: string;
};

interface NetworkInfo {
    network?: string;
    networkId?: number;
    bech32HRP: string;
    minPoWScore: number;
    localPow: boolean;
    fallbackToLocalPow: boolean;
    tipsInterval: number;
    rentStructure: {
        vByteCost: number;
        vByteFactorKey: number;
        vByteFactorData: number;
    }
}

interface MqttBrokerOptions {
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
};

interface NodeManagerBuilder {
    primaryNode?: string | Node;
    primaryPoWNode?: string | Node;
    nodes?: Array<string | Node>;
    permanodes?: Array<string | Node>;
    network?: string;
    nodeSyncEnabled: boolean;
    nodeSyncInterval: number;
    quorum: boolean;
    minQuorumSize: number;
    quorumTreshold?: boolean;
}

export interface ClientOptions {
    nodeManagerBuilder?: NodeManagerBuilder;
    brokerOptions: MqttBrokerOptions;
    networkInfo: NetworkInfo;
    apiTimeout: number;
    remotePowTimeout: number;
    offline: boolean;
    powWorkerCount: number;
}

/**
 * NodeInfo wrapper which contains the nodeinfo and the url from the node (useful when multiple nodes are used)
 */
export interface NodeInfoWrapper {
    /** The returned nodeinfo */
    nodeinfo: INodeInfo;
    /** The url from the node which returned the nodeinfo */
    url: string;
}
