package org.iota.types.events.transaction;

public enum TransactionProgressEventType {
    /// Performing input selection.
    SelectingInputs,
    /// Generating remainder value deposit address.
    GeneratingRemainderDepositAddress,
    /// Prepared transaction.
    PreparedTransaction,
    /// Prepared transaction essence hash hex encoded, required for blindsigning
    /// with a ledger nano
    PreparedTransactionEssenceHash,
    /// Signing the transaction.
    SigningTransaction,
    /// Performing PoW.
    PerformingPow,
    /// Broadcasting.
    Broadcasting,
}
