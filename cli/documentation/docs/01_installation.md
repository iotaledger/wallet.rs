# Installation

## From release

### 1. Download

Go to https://github.com/iotaledger/cli-wallet/releases and download the latest release binary for your platform.

`cli-wallet` is available on `linux`, `macos` and `windows`.

### 2. Verify checksum

Compare the checksum from the release with a checksum locally produced.

You can use the following command to produce the checksum.
```sh
shasum -a 256 [PATH TO BINARY]
```

### 3. Rename

For convenience, rename the binary to simply `wallet`.

```sh
mv [PATH TO BINARY] wallet
```

## From source

### 1. Install Rust

https://www.rust-lang.org/tools/install

### 2. Compile

```sh
git clone https://github.com/iotaledger/wallet.rs -b develop
cd wallet.rs/cli
cargo build --profile production
```

Resulting binary will be located at `./target/production/wallet`.
