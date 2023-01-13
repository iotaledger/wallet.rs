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

## 1.0.0-rc.5 - YYYY-MM-DD

### Added

- `AccountManager::get_participation_event_ids` method;
- `RequiredStorageDeposit::{alias(), basic(), foundry(), nft()}` getters;
- `TransactionOptionsDto`;

### Changed

- Updated dependencies;
- Message interface methods to accept `TransactionOptionsDto` instead of `TransactionOptions`;

### Removed
- `clear_listeners` from the `WalletMessageHandler`

## 1.0.0-rc.4 - 2022-12-23

### Added

- `RequiredStorageDeposit` and `RequiredStorageDepositDto` types;
- `account/types/balance` module;
- `AccountBuilder::with_bech32_hrp()`;
- `Account::retry_transaction_until_included()`;
- `RetryTransactionUntilIncluded` to message interface account methods;
- `AccountMethod::RequestFundsFromFaucet` to message interface;
- `FilterOptions::output_types` field;
- `{NativeTokensBalance, NativeTokensBalanceDto}::metadata` field;
- `{Account, AccountDto}::native_token_foundries` field;
- `SyncOptions::sync_native_token_foundries` field;

### Changed

- `AccountBalance::required_storage_deposit` changed from `u64` to `RequiredStorageDeposit`;
- `AccountBalanceDto::required_storage_deposit` changed from `String` to `RequiredStorageDepositDto`;
- Move all balance related types to the `account/types/balance` module;
- `AccountBalanceDto`, `BaseCoinBalanceDto` and `NativeTokensBalanceDto` moved from `message_interface/dtos` to `account/types/balance`;
- `Account::vote(), AccountMethod::Vote()` parameters are now optional to support revoting;
- Fields of `Error::{ConsolidationRequired, InsufficientFunds, InvalidCoinType}` are now named;

### Removed

- `clear_listeners` from message interface;
- `listen` from message interface;
- default bech32 HRP in account builder;
- `Copy` from `FilterOptions`;

## 1.0.0-rc.3 - 2022-11-24

### Added

- `IssuerFeature` and `SenderFeature` to `prepare_output()` options argument;
- Participation feature and functions;
- `init_logger` to message interface mod;

### Changed

- Use `OutputWithMetadataResponse` instead of `OutputResponse`;
- `AccountHandle::build_transaction_essence` made async;
- `Error::ClientError` renamed to `Error::Client` and boxed;
- `WalletEvent::SpentOutput` boxed;
- Allow null nft ids in `prepare_output()` for minting;
- `TagFeature` & `MetadataFeature` encoded as hex strings instead of utf-8 strings;
- Call `try_get_outputs_metadata` instead of `try_get_outputs` when possible (more efficient);
- `send_message` to return Option which is None when no message response is received;

### Removed

- `Error::IotaClientError` as it was a duplicate;

### Fixed

- Min storage deposit amount in `prepare_output()` with expiration unlock condition;
- NFT id in `prepare_output()` when the NFT output still has the null id;

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
