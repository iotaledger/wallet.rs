package org.iota.types;

import org.iota.types.ids.OutputId;
import org.iota.types.ids.TransactionId;

import java.util.Map;
import java.util.Set;

public class Account extends NewAbstractObject {

    /// The account index.
    private int index;
    /// The coin type.
    private int coinType;
    /// The account alias.
    private String alias;
    /// Public addresses.
    private AccountAddress[] publicAddresses;
    /// Internal addresses.
    private AccountAddress[] internalAddresses;
    /// Addresses with unspent outputs.
    private AddressWithUnspentOutputs[] addressesWithUnspentOutputs;
    /// Outputs.
    private Map<OutputId, OutputData> outputs;
    /// Unspent outputs that are currently used as input for transactions
    private Set<OutputId> lockedOutputs;
    /// Unspent outputs.
    private Map<OutputId, OutputData> unspentOutputs;
    /// Sent transactions.
    private Map<TransactionId, Transaction> transactions;
    /// Pending transactions.
    private Set<TransactionId> pendingTransactions;
    /// Incoming transactions.
    private Map<TransactionId, Map.Entry<TransactionPayload, OutputResponse[]>> incomingTransactions;

    public int getIndex() {
        return index;
    }

    public int getCoinType() {
        return coinType;
    }

    public String getAlias() {
        return alias;
    }

    public AccountAddress[] getPublicAddresses() {
        return publicAddresses;
    }

    public AccountAddress[] getInternalAddresses() {
        return internalAddresses;
    }

    public AddressWithUnspentOutputs[] getAddressesWithUnspentOutputs() {
        return addressesWithUnspentOutputs;
    }

    public Map<OutputId, OutputData> getOutputs() {
        return outputs;
    }

    public Set<OutputId> getLockedOutputs() {
        return lockedOutputs;
    }

    public Map<OutputId, OutputData> getUnspentOutputs() {
        return unspentOutputs;
    }

    public Map<TransactionId, Transaction> getTransactions() {
        return transactions;
    }

    public Set<TransactionId> getPendingTransactions() {
        return pendingTransactions;
    }

    public Map<TransactionId, Map.Entry<TransactionPayload, OutputResponse[]>> getIncomingTransactions() {
        return incomingTransactions;
    }

}