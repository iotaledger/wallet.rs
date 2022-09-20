// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

public class LedgerNanoStatus extends AbstractObject {

    /// Ledger is available and ready to be used.
    private boolean connected;
    /// Ledger is connected and locked.
    private boolean locked;
    /// Ledger blind signing enabled
    private boolean blindSigningEnabled;
    /// Ledger opened app.
    private LedgerApp app;
    /// Ledger device
    private LedgerDeviceType device;
    /// Buffer size on device
    private int bufferSize;

    public static class LedgerApp extends AbstractObject {
        /// Opened app name.
        private String name;
        /// Opened app version.
        private String version;

        public String getName() {
            return name;
        }

        public String getVersion() {
            return version;
        }
    }

    public enum LedgerDeviceType {
        /// Device Type Nano S
        LedgerNanoS,
        /// Device Type Nano X
        LedgerNanoX,
        /// Device Type Nano S Plus
        LedgerNanoSPlus,
    }

    public boolean isConnected() {
        return connected;
    }

    public boolean isLocked() {
        return locked;
    }

    public boolean isBlindSigningEnabled() {
        return blindSigningEnabled;
    }

    public LedgerApp getApp() {
        return app;
    }

    public LedgerDeviceType getDevice() {
        return device;
    }

    public int getBufferSize() {
        return bufferSize;
    }
}