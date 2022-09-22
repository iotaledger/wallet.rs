package org.iota.types;

public class Segment extends AbstractObject {
    private boolean hardened;
    private UnsignedByte[] bs;

    public Segment(boolean hardened, UnsignedByte[] bs) {
        this.hardened = hardened;
        this.bs = bs;
    }

    public boolean isHardened() {
        return hardened;
    }

    public UnsignedByte[] getBs() {
        return bs;
    }
}