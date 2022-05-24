import type { INodeInfo, IRent } from '@iota/types';

export enum Network {
    Mainnet,
    Testnet,
}

export type Auth = {
    jwt?: string;
    username?: string;
    password?: string;
};

interface NetworkInfo {
    network?: string;
    networkId?: number;
    bech32HRP?: string;
    minPoWScore?: number;
    localPow?: boolean;
    fallbackToLocalPow?: boolean;
    tipsInterval?: number;
    rentStructure?: IRent
}

export type Node = {
    url: string;
    auth?: Auth;
    disabled?: boolean;
};

export interface ClientOptions {
    apiTimeout?: number;
    automaticDisconnect?: boolean;
    maxReconnectionAttempts?: number;
    minQuorumSize?: number;
    network?: string;
    networkInfo?: NetworkInfo;
    nodes?: Array<string | Node>;
    nodeSyncEnabled?: boolean;
    nodeSyncInterval?: number;
    offline?: boolean;
    permanodes?: Array<string | Node>;
    port?: number;
    powWorkerCount?: number;
    primaryNode?: string | Node;
    primaryPoWNode?: string | Node;
    quorum?: boolean;
    quorumTreshold?: boolean;
    remotePowTimeout?: number;
    // timeout in seconds
    timeout?: number;
    useWs?: boolean;
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
