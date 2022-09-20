package org.iota.types;

public class Segment extends AbstractObject {
    private boolean hardened;
    private byte[] bs;

    public Segment(boolean hardened, byte[] bs) {
        this.hardened = hardened;
        this.bs = bs;
    }

    public boolean isHardened() {
        return hardened;
    }

    public byte[] getBs() {
        return bs;
    }
}
