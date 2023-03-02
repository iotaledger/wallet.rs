# IOTA Wallet Library - Swift binding

Swift binding to the IOTA wallet library

## Requirements

Ensure you have first installed the latest stable version of Rust and Cargo.

## Installation

For current system architecture
```
cd wallet.rs/bindings/swift
cargo build
```

For debug build, copy `wallet.rs/bindings/swift/iota_wallet_ffi.h` and `wallet.rs/bindings/swift/target/debug/libiota_wallet.dylib` to `wallet.rs/bindings/swift/xcode/IotaWallet/iota_wallet`.

Open and `build wallet.rs/bindings/swift/xcode/IotaWallet/IotaWallet.xcodeproj`. The xcode build product is an Objective-C framework that can be used in Swift.

