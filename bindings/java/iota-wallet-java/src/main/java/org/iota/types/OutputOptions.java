package org.iota.types;

public class OutputOptions extends NewAbstractObject {

    private String recipientAddress;
    private String amount;
    private Assets assets;
    private Features features;
    private Unlocks unlocks;
    private StorageDeposit storageDeposit;

    public static class Assets extends NewAbstractObject {
        private NativeToken[] nativeTokens;
        private String nftId;
    }

    public static class Features extends NewAbstractObject {
        private String tag;
        private String metadata;
    }

    public static class Unlocks extends NewAbstractObject {
        private int expirationUnixTime;
        private int timelockUnixTime;
    }

    public static class StorageDeposit extends NewAbstractObject {
        private ReturnStrategy returnStrategy;
        // If account has 2 Mi, min storage deposit is 1 Mi and one wants to send 1.5 Mi, it wouldn't be possible with a
        // 0.5 Mi remainder. To still send a transaction, the 0.5 can be added to the output automatically, if set to true
        private boolean useExcessIfLow;
    }

    public enum ReturnStrategy {
        // A storage deposit return unlock condition will be added with the required minimum storage deposit
        Return,
        // The recipient address will get the additional amount to reach the minimum storage deposit gifted
        Gift,
    }

}