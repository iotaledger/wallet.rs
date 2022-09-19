package org.iota.types.unlocks;

import org.iota.types.signature.Signature;

public class SignatureUnlock extends Unlock {
    private int type = 0;
    private Signature signature;

    public SignatureUnlock(Signature signature) {
        this.signature = signature;
    }

    public int getType() {
        return type;
    }

    public Signature getSignature() {
        return signature;
    }

}