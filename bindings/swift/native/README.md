# IOTA Wallet Library - Swift binding

Swift binding to the IOTA wallet library

## Requirements

Ensure you have first installed the latest stable version of Rust and Cargo.

## Installation

```
cd wallet.rs/bindings/swift/native
./make_universal_xcframework.sh
```

Or for a single architecture:

```
./make_framework.sh --target aarch64-apple-darwin
./make_framework.sh --target aarch64-apple-ios
./make_framework.sh --target x86_64-apple-ios
```

The result can be found in `wallet.rs/bindings/swift/native/target/<ARCH>`.