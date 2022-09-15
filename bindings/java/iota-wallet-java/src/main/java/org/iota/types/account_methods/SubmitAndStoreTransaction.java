package org.iota.types.account_methods;

import org.iota.types.SignedTransactionData;

/// Validate the transaction, submit it to a node and store it in the account.
public class SubmitAndStoreTransaction implements AccountMethod {

    private SignedTransactionData signedTransactionData;

    public SubmitAndStoreTransaction withSignedTransactionData(SignedTransactionData signedTransactionData) {
        this.signedTransactionData = signedTransactionData;
        return this;
    }
}