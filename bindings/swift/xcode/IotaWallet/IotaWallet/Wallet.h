// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#import <Foundation/Foundation.h>

@interface Wallet : NSProxy

NS_ASSUME_NONNULL_BEGIN

typedef void (^WalletHandler) (NSString * _Nullable message, NSError * _Nullable error);

- (nullable instancetype) init;
- (nullable instancetype) initWithStoragePath:(NSString *) path;
- (void) sendMessage:(NSString *) message completion: (WalletHandler) completion;

NS_ASSUME_NONNULL_END

@end

