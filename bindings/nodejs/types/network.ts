import type { INodeInfo } from '@iota/types';

export enum Network {
    Mainnet,
    Testnet,
}

export type Auth = {
    jwt?: string;
    username?: string;
    password?: string;
};

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
};

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

/**
 * NodeInfo wrapper which contains the nodeinfo and the url from the node (useful when multiple nodes are used)
 */
export interface NodeInfoWrapper {
    /** The returned nodeinfo */
    nodeinfo: INodeInfo;
    /** The url from the node which returned the nodeinfo */
    url: string;
}
