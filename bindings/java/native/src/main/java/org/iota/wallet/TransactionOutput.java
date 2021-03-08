// Automatically generated by flapigen
package org.iota.wallet;


public final class TransactionOutput {
    @Override
    public String toString() {
        return this.to_string();
    }


    private TransactionOutput() {}

    private final String to_string() {
        String ret = do_to_string(mNativeObj);

        return ret;
    }
    private static native String do_to_string(long self);

    public final OutputKind kind() {
        int ret = do_kind(mNativeObj);
        OutputKind convRet = OutputKind.fromInt(ret);

        return convRet;
    }
    private static native int do_kind(long self);

    public synchronized void delete() {
        if (mNativeObj != 0) {
            do_delete(mNativeObj);
            mNativeObj = 0;
       }
    }
    @Override
    protected void finalize() throws Throwable {
        try {
            delete();
        }
        finally {
             super.finalize();
        }
    }
    private static native void do_delete(long me);
    /*package*/ TransactionOutput(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}