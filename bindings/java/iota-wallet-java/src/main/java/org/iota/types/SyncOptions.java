package org.iota.types;

public class SyncOptions extends AbstractObject {
    /// Specific Bech32 encoded addresses of the account to sync, if addresses are provided, then `address_start_index`
    /// will be ignored
    private String[] addresses;
    /// Address index from which to start syncing addresses. 0 by default, using a higher index will be faster because
    /// addresses with a lower index will be skipped, but could result in a wrong balance for that reason
    private int addressStartIndex;
    /// Address index from which to start syncing internal addresses. 0 by default, using a higher index will be faster
    /// because addresses with a lower index will be skipped, but could result in a wrong balance for that reason
    private int addressStartIndexInternal;
    /// Usually syncing is skipped if it's called in between 200ms, because there can only be new changes every
    /// milestone and calling it twice "at the same time" will not return new data
    /// When this to true, we will sync anyways, even if it's called 0ms after the las sync finished.
    private boolean forceSyncing;
    /// Try to sync transactions from incoming outputs with their inputs. Some data may not be obtained if it has been
    /// pruned.
    private boolean syncIncomingTransactions;
    /// Checks pending transactions and promotes/reattaches them if necessary.
    private boolean syncPendingTransactions;
    /// Specifies if only basic outputs should be synced or also alias and nft outputs
    private boolean syncAliasesAndNfts;
    /// Specifies if only basic outputs with an AddressUnlockCondition alone should be synced, will overwrite
    private boolean syncOnlyMostBasicOutputs;

    public String[] getAddresses() {
        return addresses;
    }

    public SyncOptions withAddresses(String[] addresses) {
        this.addresses = addresses;
        return this;
    }

    public int getAddressStartIndex() {
        return addressStartIndex;
    }

    public SyncOptions withAddressStartIndex(int addressStartIndex) {
        this.addressStartIndex = addressStartIndex;
        return this;
    }

    public int getAddressStartIndexInternal() {
        return addressStartIndexInternal;
    }

    public SyncOptions withAddressStartIndexInternal(int addressStartIndexInternal) {
        this.addressStartIndexInternal = addressStartIndexInternal;
        return this;
    }

    public boolean isForceSyncing() {
        return forceSyncing;
    }

    public SyncOptions withForceSyncing(boolean forceSyncing) {
        this.forceSyncing = forceSyncing;
        return this;
    }

    public boolean isSyncIncomingTransactions() {
        return syncIncomingTransactions;
    }

    public SyncOptions withSyncIncomingTransactions(boolean syncIncomingTransactions) {
        this.syncIncomingTransactions = syncIncomingTransactions;
        return this;
    }

    public boolean isSyncPendingTransactions() {
        return syncPendingTransactions;
    }

    public SyncOptions withSyncPendingTransactions(boolean syncPendingTransactions) {
        this.syncPendingTransactions = syncPendingTransactions;
        return this;
    }

    public boolean isSyncAliasesAndNfts() {
        return syncAliasesAndNfts;
    }

    public SyncOptions withSyncAliasesAndNfts(boolean syncAliasesAndNfts) {
        this.syncAliasesAndNfts = syncAliasesAndNfts;
        return this;
    }

    public boolean isSyncOnlyMostBasicOutputs() {
        return syncOnlyMostBasicOutputs;
    }

    public SyncOptions withSyncOnlyMostBasicOutputs(boolean syncOnlyMostBasicOutputs) {
        this.syncOnlyMostBasicOutputs = syncOnlyMostBasicOutputs;
        return this;
    }
}