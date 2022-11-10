# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security -->

## 1.0.0-rc.3 - 20XX-XX-XX

### Added

- `IssuerFeature` and `SenderFeature` to `prepare_output()` options argument;

### Changed

- Use `OutputWithMetadataResponse` instead of `OutputResponse`;
- `AccountHandle::build_transaction_essence` made async;
- `Error::ClientError` renamed to `Error::Client` and boxed;
- `WalletEvent::SpentOutput` boxed;
- Allow null nft ids in `prepare_output()` for minting;
- `TagFeature` & `MetadataFeature` encoded as hex strings instead of utf-8 strings;
- Call `try_get_outputs_metadata` instead of `try_get_outputs` when possible (more efficient);

### Removed

- `Error::IotaClientError` as it was a duplicate;

### Fixed

- min storage deposit amount in `prepare_output()` with expiration unlock condition;

## 1.0.0-rc.2 - 2022-10-28

### Added

- `GetAccountIndexes` and `AccountIndexes` to message interface;
- Missing serde rename for the `returnAddress` field of `AddressWithMicroAmountDto`;
- Check for parameters before creating the database;
- Transaction essence and payload length validation;

### Changed

- Updated dependencies;
- Derived `Serialize` for `Message` and `AccountMethod` instead of custom impl;
- Removed `AccountToCreate` struct and moved its field to `Message::CreateAccount`;

### Fixed

- NFT output claiming;
- Alias and NFT owned outputs syncing;
- Alias or nft addresses in address unlock condition;

## 1.0.0-rc.1 - 2022-09-28

First RC for the stardust implementation of the wallet.
