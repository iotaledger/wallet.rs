// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;
public class ClientConfig extends AbstractObject {
    private String primaryNode;
    private String primaryPowNode;
    private String[] nodes;
    private String[] permanodes;
    private Boolean nodeSyncEnabled;
    private NodeSyncInterval nodeSyncInterval;
    private Boolean quorum;
    private Integer minQuorumSize;
    private Integer quorumThreshold;
    private String network;
    private String networkId;
    private String bech32Hrp;
    private Double minPowScore;
    private Boolean localPow;
    private Boolean fallbackToLocalPow;
    private Integer tipsInterval;
    private RentStructure rentStructure;
    private ApiTimeout apiTimeout;
    private RemotePowTimeout remotePowTimeout;
    private Boolean offline;
    private Integer powWorkerCount;

    static class NodeSyncInterval {
        private int secs;
        private int nanos;

        public NodeSyncInterval withSecs(int secs) {
            this.secs = secs;
            return this;
        }

        public NodeSyncInterval withNanos(int nanos) {
            this.nanos = nanos;
            return this;
        }
    }

    static class RentStructure {
        private int vByteCost;
        private int vByteFactorKey;
        private int vByteFactorData;

        public RentStructure withvByteCost(int vByteCost) {
            this.vByteCost = vByteCost;
            return this;
        }

        public RentStructure withvByteFactorKey(int vByteFactorKey) {
            this.vByteFactorKey = vByteFactorKey;
            return this;
        }

        public RentStructure withvByteFactorData(int vByteFactorData) {
            this.vByteFactorData = vByteFactorData;
            return this;
        }
    }

    static class ApiTimeout {
        private int secs;
        private int nanos;

        public ApiTimeout withSecs(int secs) {
            this.secs = secs;
            return this;
        }

        public ApiTimeout withNanos(int nanos) {
            this.nanos = nanos;
            return this;
        }
    }

    static class RemotePowTimeout {
        private int secs;
        private int nanos;

        public RemotePowTimeout withSecs(int secs) {
            this.secs = secs;
            return this;
        }

        public RemotePowTimeout withNanos(int nanos) {
            this.nanos = nanos;
            return this;
        }
    }


    public ClientConfig withPrimaryNode(String primaryNode) {
        this.primaryNode = primaryNode;
        return this;
    }

    public ClientConfig withPrimaryPowNode(String primaryPowNode) {
        this.primaryPowNode = primaryPowNode;
        return this;
    }

    public ClientConfig withNodes(String[] nodes) {
        this.nodes = nodes;
        return this;
    }

    public ClientConfig withPermanodes(String[] permanodes) {
        this.permanodes = permanodes;
        return this;
    }

    public ClientConfig withNodeSyncEnabled(Boolean nodeSyncEnabled) {
        this.nodeSyncEnabled = nodeSyncEnabled;
        return this;
    }

    public ClientConfig withNodeSyncInterval(NodeSyncInterval nodeSyncInterval) {
        this.nodeSyncInterval = nodeSyncInterval;
        return this;
    }

    public ClientConfig withQuorum(Boolean quorum) {
        this.quorum = quorum;
        return this;
    }

    public ClientConfig withMinQuorumSize(Integer minQuorumSize) {
        this.minQuorumSize = minQuorumSize;
        return this;
    }

    public ClientConfig withQuorumThreshold(Integer quorumThreshold) {
        this.quorumThreshold = quorumThreshold;
        return this;
    }

    public ClientConfig withNetwork(String network) {
        this.network = network;
        return this;
    }

    public ClientConfig withNetworkId(String networkId) {
        this.networkId = networkId;
        return this;
    }

    public ClientConfig withBech32Hrp(String bech32Hrp) {
        this.bech32Hrp = bech32Hrp;
        return this;
    }

    public ClientConfig withMinPowScore(Double minPowScore) {
        this.minPowScore = minPowScore;
        return this;
    }

    public ClientConfig withLocalPow(Boolean localPow) {
        this.localPow = localPow;
        return this;
    }

    public ClientConfig withFallbackToLocalPow(Boolean fallbackToLocalPow) {
        this.fallbackToLocalPow = fallbackToLocalPow;
        return this;
    }

    public ClientConfig withTipsInterval(Integer tipsInterval) {
        this.tipsInterval = tipsInterval;
        return this;
    }

    public ClientConfig withRentStructure(RentStructure rentStructure) {
        this.rentStructure = rentStructure;
        return this;
    }

    public ClientConfig withApiTimeout(ApiTimeout apiTimeout) {
        this.apiTimeout = apiTimeout;
        return this;
    }

    public ClientConfig withRemotePowTimeout(RemotePowTimeout remotePowTimeout) {
        this.remotePowTimeout = remotePowTimeout;
        return this;
    }

    public ClientConfig withOffline(Boolean offline) {
        this.offline = offline;
        return this;
    }

    public ClientConfig withPowWorkerCount(Integer powWorkerCount) {
        this.powWorkerCount = powWorkerCount;
        return this;
    }
}
