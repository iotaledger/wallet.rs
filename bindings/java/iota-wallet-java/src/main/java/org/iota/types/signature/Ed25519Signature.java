package org.iota.types.signature;

public class Ed25519Signature extends Signature {

    private int type = 0;
    private String publicKey;
    private String signature;

    public Ed25519Signature(String publicKey, String signature) {
        this.publicKey = publicKey;
        this.signature = signature;
    }

    public String getPublicKey() {
        return publicKey;
    }

    public String getSignature() {
        return signature;
    }
}