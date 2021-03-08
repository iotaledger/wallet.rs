package org.iota.wallet;

public class WalletException extends RuntimeException {

    public WalletException() {
        super();
    }

    public WalletException(String errorMessage) {
        super(errorMessage);
    }
}
